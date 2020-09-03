#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};
use std::iter::{FromIterator, Peekable, self};
use std::mem::replace;
use std::ops::Add;
use std::process::exit;
use std::str::FromStr;
use std::time::{Duration, Instant};

use joinery::prelude::*;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::{self, Regex};

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

        let (solution, duration) = timed(move || solve(input.trim()));
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

#[derive(Debug, Clone, Default)]
struct Node {
    children: Vec<Node>,
    meta: Vec<usize>,
}

impl Node {
    fn meta_score(&self) -> usize {
        self.meta.iter().sum::<usize>() +
        self.children.iter().map(|child| child.meta_score()).sum::<usize>()
    }

    fn build(iter: &mut impl Iterator<Item=usize>) -> Self {
        let num_children = iter.next().unwrap();
        let num_meta = iter.next().unwrap();

        let children = iter::repeat_with(|| Node::build(iter)).take(num_children).collect();
        let meta = iter.take(num_meta).collect();

        Node {
            children,
            meta,
        }
    }

    fn value(&self) -> usize {
        if self.children.is_empty() {
            self.meta.iter().sum()
        } else {
            self.meta
                .iter()
                .filter_map(move |meta| {
                    let index = meta.checked_sub(1)?;
                    let child = self.children.get(index)?;
                    Some(child.value())
                })
                .sum()
        }
    }
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let data = Node::build(&mut input
        .split_whitespace()
        .map(|part| part.parse().unwrap())
    );

    data.value()
}

