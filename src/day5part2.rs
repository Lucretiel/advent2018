#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};
use std::iter::{FromIterator, Peekable};
use std::mem::replace;
use std::ops::Add;
use std::process::exit;
use std::str::FromStr;
use std::time::{Duration, Instant};

use joinery::prelude::*;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::{self, Regex};

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
struct Coords {
    x: i32,
    y: i32,
}

impl Coords {
    fn distance(&self, rhs: Coords) -> i32 {
        (self.x - rhs.x).abs() + (self.y - rhs.y).abs()
    }
}

#[inline(always)]
fn solve(input: String) -> impl Display {
    let pattern = Regex::new(r"(\d+), (\d+)").unwrap();
    let coords: Vec<Coords> = pattern
        .captures_iter(&input)
        .map(|caps| Coords {
            x: caps.parse(1),
            y: caps.parse(2),
        })
        .collect();

    let coords_ref = &coords;

    (-500..1000).into_par_iter().flat_map(|x| {
        (-500..1000).into_par_iter().map(move |y| {
            coords_ref.iter().map(|coord| coord.distance(Coords{x, y})).sum()
        })
    })
    .filter(|score: &i32| *score < 10000)
    .count()
}

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
        let mut input = io::stdin().read_string().unwrap_or_else(|err| {
            eprintln!("Error reading input: {}", err);
            exit(1);
        });

        input.truncate(input.trim_end().len());

        let (solution, duration) = timed(move || solve(input));
        println!("{}", solution);

        eprintln!("Algorithm duration: {:?}", duration);
    });
    eprintln!("Total duration: {:?}", total_duration);
}
