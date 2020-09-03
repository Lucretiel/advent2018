#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};
use std::iter::{self, FromIterator, Peekable, FusedIterator};
use std::mem::replace;
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

#[derive(Debug, Clone)]
struct UniqueIterator<I: Iterator> {
    last: Option<I::Item>,
    iter: I,
}

impl<I: Iterator> Iterator for UniqueIterator<I>
    where I::Item: Clone + PartialEq,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match &self.last {
            None =>  self.iter.next(),
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

const serial: i64 = 7347;

const fn power_of(x: i64, y: i64) -> i64 {
    ((((((x + 10) * y) + serial) * (x + 10)) % 1000) / 100) - 5
}

fn multi_power_of(x: i64, y: i64, size:i64) -> i64 {
    (0..size).flat_map(move |dx|
        (0..size).map(move |dy|
            power_of(x + dx, y + dy)
        )
    ).sum()
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    (2..100).for_each(move |size| {
        let (x, y, score) = (1 ..= 300 - size).flat_map(move |x|
            (1 ..= 300 - size).map(move |y|
                (x, y, multi_power_of(x, y, size))
            )
        ).max_by_key(|(_x, _y, score)| *score).unwrap();

        println!("{},{},{}: {}", x, y, size, score)
    });

    "DONE"
}
