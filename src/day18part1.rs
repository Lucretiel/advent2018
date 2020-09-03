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

use joinery::prelude::*;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::{self, Regex};
use gridly::prelude::*;
use gridly_grids::*;
use generations::*;
use lazy_format::lazy_format;

// DON'T TOUCH THIS
#[inline(always)]
fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let result = f();
    let end = Instant::now();
    (result, end - start)
}

trait ReadString: Read {
    fn read_string(&mut self) -> io::Result<String> {
        let mut data = String::new();
        self.read_to_string(&mut data).map(|_| data)
    }
}

impl<T: Read> ReadString for T {}

fn main() {
    let ((), total_duration) = timed(move || {
        let input = io::stdin().read_string().unwrap_or_else(|err| {
            eprintln!("Error reading input: {}", err);
            exit(1);
        });

        let (solution, duration) = timed(move || solve(&input));
        println!("{}", solution);

        eprintln!("Algorithm duration: {:?}", duration);
    });
    eprintln!("Total duration: {:?}", total_duration);
}

trait RegexExtractor<'t> {
    fn field<T>(&self, index: usize) -> T
    where
        &'t str: Into<T>;

    fn parse<T: FromStr>(&self, index: usize) -> T
    where
        T::Err: Display;
}

impl<'t> RegexExtractor<'t> for regex::Captures<'t> {
    #[inline]
    fn field<T>(&self, index: usize) -> T
    where
        &'t str: Into<T>,
    {
        self.get(index)
            .unwrap_or_else(move || panic!("Group {} didn't match anything", index))
            .as_str()
            .into()
    }

    #[inline]
    fn parse<T: FromStr>(&self, index: usize) -> T
    where
        T::Err: Display,
    {
        let field: &str = self.field(index);

        field.parse().unwrap_or_else(move |err| {
            panic!("Failed to parse group {} \"{}\": {}", index, field, err)
        })
    }
}
// CODE GOES HERE

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Acre {
    Empty,
    Tree,
    Lumberyard,
}

impl Default for Acre {
    fn default() -> Self {
        Acre::Empty
    }
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let mut grid: VecGrid<Acre> = VecGrid::new(Rows(50) + Columns(50)).unwrap();
    let grid2 = grid.clone();

    for (row, line) in (0..50).map(Row).zip(input.lines()) {
        for (col, c) in (0..50).map(Column).zip(line.trim().chars()) {
            grid[(row, col)] = match c {
                '.' => Acre::Empty,
                '|' => Acre::Tree,
                '#' => Acre::Lumberyard,
                _ => panic!("Uncrecognized character: {:?}", c)
            };
        }
    }

    let gens = Generations::new(grid, grid2);
    let mut sim = gens.with_rule(move |current_gen, next_gen| {
        current_gen.row_range().cross(current_gen.column_range()).for_each(move |loc| {
            use Acre::*;

            let adjacent_cells = TOUCHING_ADJACENCIES.iter().filter_map(|v| current_gen.get(loc + v).ok());

            next_gen[loc] = match current_gen[loc] {
                Empty => if adjacent_cells.filter(|&&c| c == Tree).count() >= 3 {Tree} else {Empty},
                Tree => if adjacent_cells.filter(|&&c| c == Lumberyard).count() >= 3 { Lumberyard} else {Tree},
                Lumberyard => {
                    let mut tree = false;
                    let mut lumber = false;
                    for cell in adjacent_cells {
                        match cell {
                            Lumberyard => lumber = true,
                            Tree => tree = true,
                            _ => {}
                        };
                    }
                    if tree && lumber {Lumberyard} else {Empty}
                }
            };
        });
    });

    for i in 0..1000000000 {
        sim.step();

        if i % 100 == 0 {
            let current = sim.current();
            let print = lazy_format!("{row}\n" for row in current.rows().iter().map(|row|
                lazy_format!("{cell}" for cell in row.iter().map(|c| match c {
                    Acre::Lumberyard => "#",
                    Acre::Tree => "|",
                    Acre::Empty => ".",
                }))));

            println!("{}", print);
        }
    }

    let mut trees = 0;
    let mut lumber = 0;

    for row in sim.current().rows().iter() {
        for cell in row.iter() {
            match cell {
                Acre::Tree => trees += 1,
                Acre::Lumberyard => lumber += 1,
                Acre::Empty => {},
            }
        }
    }

    trees * lumber
}
