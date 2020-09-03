#![allow(unused_imports)]

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::io::{self, Read};
use std::iter::{self, FromIterator, FusedIterator, Peekable};
use std::mem::{replace, swap};
use std::ops::Add;
use std::process::exit;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};

use gridly::prelude::*;
use joinery::prelude::*;
use lazy_static::lazy_static;
use rayon::prelude::*;
use lazy_format::prelude::*;
use regex::{self, Regex};

trait FromCode: Sized {
    fn from_code(code: usize) -> Option<Self>;
}

impl FromCode for usize {
    #[inline]
    fn from_code(code: usize) -> Option<Self> {
        Some(code)
    }
}

trait RegFetch {
    fn get_from(&self, registers: &Registers) -> usize;
}

impl RegFetch for usize {
    #[inline]
    fn get_from(&self, _reg: &Registers) -> usize {
        *self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RegisterID {
    A,
    B,
    C,
    D,
}

impl FromCode for RegisterID {
    #[inline]
    fn from_code(code: usize) -> Option<Self> {
        use RegisterID::*;

        match code {
            0 => Some(A),
            1 => Some(B),
            2 => Some(C),
            3 => Some(D),
            _ => None,
        }
    }
}

impl FromStr for RegisterID {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, &'static str> {
        use RegisterID::*;

        match s {
            "0" => Ok(A),
            "1" => Ok(B),
            "2" => Ok(C),
            "3" => Ok(D),
            _ => Err("Invalid register id"),
        }
    }
}

impl RegFetch for RegisterID {
    #[inline]
    fn get_from(&self, registers: &Registers) -> usize {
        registers.get(*self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Instruction {
    opcode: usize,
    inputA: usize,
    inputB: usize,
    output: RegisterID,
}

impl FromStr for Instruction {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Instruction, &'static str> {
        let mut parts = input.split_whitespace();
        Ok(Instruction {
            opcode: parts
                .next()
                .ok_or("Incorrect number of operands")?
                .parse()
                .map_err(|_| "Failed to parse opcode")?,
            inputA: parts
                .next()
                .ok_or("Incorrect number of operands")?
                .parse()
                .map_err(|_| "Failed to parse inputA")?,
            inputB: parts
                .next()
                .ok_or("Incorrect number of operands")?
                .parse()
                .map_err(|_| "Failed to parse inputB")?,
            output: parts
                .next()
                .ok_or("Incorrect number of operands")?
                .parse()?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Params<InputA: RegFetch + FromCode, InputB: RegFetch + FromCode> {
    inputA: InputA,
    inputB: InputB,
    output: RegisterID,
}

impl<InputA: RegFetch + FromCode, InputB: RegFetch + FromCode> Params<InputA, InputB> {
    fn from_instruction(instruction: &Instruction) -> Option<Self> {
        Some(Self {
            inputA: InputA::from_code(instruction.inputA)?,
            inputB: InputB::from_code(instruction.inputB)?,
            output: instruction.output,
        })
    }

    #[inline]
    fn apply(&self, registers: &mut Registers, op: impl Fn(usize, usize) -> usize) {
        registers.set(
            self.output,
            op(
                self.inputA.get_from(registers),
                self.inputB.get_from(registers),
            ),
        );
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Operation {
    Addr(Params<RegisterID, RegisterID>),
    Addi(Params<RegisterID, usize>),

    Mulr(Params<RegisterID, RegisterID>),
    Muli(Params<RegisterID, usize>),

    Banr(Params<RegisterID, RegisterID>),
    Bani(Params<RegisterID, usize>),

    Borr(Params<RegisterID, RegisterID>),
    Bori(Params<RegisterID, usize>),

    Setr(Params<RegisterID, RegisterID>),
    Seti(Params<usize, RegisterID>),

    Gtir(Params<usize, RegisterID>),
    Gtri(Params<RegisterID, usize>),
    Gtrr(Params<RegisterID, RegisterID>),

    Eqir(Params<usize, RegisterID>),
    Eqri(Params<RegisterID, usize>),
    Eqrr(Params<RegisterID, RegisterID>),
}

impl Operation {
    fn from_instruction(instruction: &Instruction) -> Option<Self> {
        use Operation::*;

        match instruction.opcode {
            0 => Some(Gtrr(Params::from_instruction(instruction)?)),
            1 => Some(Borr(Params::from_instruction(instruction)?)),
            2 => Some(Gtir(Params::from_instruction(instruction)?)),
            3 => Some(Eqri(Params::from_instruction(instruction)?)),
            4 => Some(Addr(Params::from_instruction(instruction)?)),
            5 => Some(Seti(Params::from_instruction(instruction)?)),
            6 => Some(Eqrr(Params::from_instruction(instruction)?)),
            7 => Some(Gtri(Params::from_instruction(instruction)?)),
            8 => Some(Banr(Params::from_instruction(instruction)?)),
            9 => Some(Addi(Params::from_instruction(instruction)?)),
            10 => Some(Setr(Params::from_instruction(instruction)?)),
            11 => Some(Mulr(Params::from_instruction(instruction)?)),
            12 => Some(Bori(Params::from_instruction(instruction)?)),
            13 => Some(Muli(Params::from_instruction(instruction)?)),
            14 => Some(Eqir(Params::from_instruction(instruction)?)),
            15 => Some(Bani(Params::from_instruction(instruction)?)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct Registers(usize, usize, usize, usize);

macro_rules! apply_block {
    ($this:ident match $input:ident {
        $(
            $($Op:ident)* => |$a:ident, $b:ident| $body:expr,
        )*
    }) => {
        match $input {
            $($(
                Operation::$Op(params) => params.apply($this, #[inline] |$a, $b| $body),
            )*)*
        }
    }
}

impl Registers {
    #[inline]
    fn get(&self, id: RegisterID) -> usize {
        match id {
            RegisterID::A => self.0,
            RegisterID::B => self.1,
            RegisterID::C => self.2,
            RegisterID::D => self.3,
        }
    }

    #[inline]
    fn set(&mut self, id: RegisterID, value: usize) {
        match id {
            RegisterID::A => self.0 = value,
            RegisterID::B => self.1 = value,
            RegisterID::C => self.2 = value,
            RegisterID::D => self.3 = value,
        };
    }

    #[inline]
    fn exec(&mut self, op: &Operation) {
        apply_block! {
            self match op {
                Addr Addi => |a, b| a + b,
                Mulr Muli => |a, b| a * b,
                Banr Bani => |a, b| (a & b),
                Borr Bori => |a, b| (a | b),
                Setr Seti => |a, _b| a,
                Gtri Gtir Gtrr => |a, b| if a > b {1} else {0},
                Eqri Eqir Eqrr => |a, b| if a == b {1} else {0},
            }
        }
    }
}

impl FromStr for Registers {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Registers, &'static str> {
        let mut split = s.split(", ");
        Ok(Registers(
            split
                .next()
                .ok_or("Incorrect number of register values")?
                .parse()
                .map_err(|_| "Invalid register value")?,
            split
                .next()
                .ok_or("Incorrect number of register values")?
                .parse()
                .map_err(|_| "Invalid register value")?,
            split
                .next()
                .ok_or("Incorrect number of register values")?
                .parse()
                .map_err(|_| "Invalid register value")?,
            split
                .next()
                .ok_or("Incorrect number of register values")?
                .parse()
                .map_err(|_| "Invalid register value")?,
        ))
    }
}

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read stdin");

    let mut split = input.split("\n\n\n\n");
    let part1 = split.next().unwrap();
    let part2 = split.next().unwrap();

    let mut registers = Registers::default();
    part2.trim().lines()
        .map(|line| Instruction::from_str(line.trim()).unwrap_or_else(|_| panic!("Failed to parse instruction from {}", line.trim())))
        .map(|inst| Operation::from_instruction(&inst).unwrap_or_else(|| panic!("Failed to convert {:?} to operation", inst)))
        .for_each(|op| registers.exec(&op));

    println!("{:?}", registers)
}
