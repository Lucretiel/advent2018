#![allow(unused_imports)]

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

#[derive(Debug, Clone)]
struct UniqueIterator<I: Iterator> {
    last: Option<I::Item>,
    iter: I,
}

impl<I: Iterator> Iterator for UniqueIterator<I>
where
    I::Item: Clone + PartialEq,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match &self.last {
            None => self.iter.next(),
            Some(last) => self.iter.find(|item| item != last),
        };

        self.last = next.clone();
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.iter.size_hint();

        if self.last.is_none() && min > 0 {
            (1, max)
        } else {
            (0, max)
        }
    }
}

impl<I: FusedIterator> FusedIterator for UniqueIterator<I> where I::Item: Clone + PartialEq {}

trait BetterIterator: Iterator + Sized {
    fn unique(self) -> UniqueIterator<Self> {
        UniqueIterator {
            iter: self,
            last: None,
        }
    }
}

impl<I: Iterator> BetterIterator for I {}

// CODE GOES HERE

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Cell {
    Empty,
    Clay,
    FlowWater,
    StableWater,
}

use Cell::*;

fn flow_down<G: GridMut<Item=Cell>>(grid: &mut G, mut loc: Location) -> usize {
    let mut count = 0;

    loop {
        let next = loc + Down;
        match grid.get_mut(next) {
            Err(_) | Ok(FlowWater) => return count,
            Ok(Clay) | Ok(StableWater) => break,
            Ok(cell @ Empty) => {
                loc = next;
                count += 1;
                *cell = FlowWater;
            }
        }
    }

    // We're at the bottom. Flow left and right, then return.
    count + flow_left(grid, loc) + flow_right(grid, loc)
}

fn flow_left<G: Grid>(grid: &mut G, loc: Location) -> usize {
    let mut count = 0;
    let

    loop {
        let next = loc + Left;
        match grid.get_mut(next) {
            Err(_) => panic!("Flowed off left side"),
            Ok(FlowWater) => return count,
            Ok(Cell @ Empty)
        }
    }
}

fn flow_right<G: Grid>(grid: &mut G, loc: Location) -> usize {
    10
}


#[inline(always)]
fn solve(input: &str) -> impl Display {
    let mut grid = VecGrid::new_fill((2000, 2000), &Empty).unwrap();

    let pattern = Regex::new(r"^(:x?
        (?P<vertical>  x=(?P<col>\d+), \s+ y=(?P<min_row>\d+) \.\. (?P<max_row>\d+))|
        (?P<horizontal>y=(?P<row>\d+), \s+ x=(?P<min_col>\d+) \.\. (?P<max_col>\d+))
    )$").unwrap();

    for line in input.lines().map(|line| line.trim()) {
        let caps = pattern.captures(line).unwrap();

        if caps.name("vertical").is_some() {
            let range = LocationRange::bounded(
                Column(caps["col"].parse().unwrap()),
                Row(caps["min_row"].parse().unwrap()),
                Row(caps["max_row"].parse::<isize>().unwrap() + 1),
            );
            range.for_each(|loc| {
                grid.set(loc, Cell::Clay).unwrap()
            })
        } else if caps.name("horizontal").is_some() {
            let range = LocationRange::bounded(
                Row(caps["row"].parse().unwrap()),
                Column(caps["min_col"].parse().unwrap()),
                Column(caps["max_col"].parse::<isize>().unwrap() + 1),
            );
            range.for_each(|loc| {
                grid.set(loc, Cell::Clay).unwrap()
            })
        }
    }

    10
}
