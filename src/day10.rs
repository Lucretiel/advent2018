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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    y: i64,
    x: i64,

    dx: i64,
    dy: i64,
}

impl Point {
    fn advance(&mut self) {
        self.x += self.dx;
        self.y += self.dy;
    }
}

#[derive(Debug, Clone)]
struct Points(Vec<Point>);

impl FromIterator<Point> for Points {
    fn from_iter<I: IntoIterator<Item = Point>>(iter: I) -> Self {
        Points(FromIterator::from_iter(iter))
    }
}

impl Display for Points {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let min_y = self.0.first().unwrap().y;
        let min_x = self.0.iter().map(|p| p.x).min().unwrap();

        let mut y = min_y;
        let mut x = min_x;

        for point in &self.0 {
            // Advance row
            if point.y > y {
                x = min_x;

                while point.y > y {
                    y += 1;
                    '\n'.fmt(f)?;
                }
            }

            // Advance columns
            while point.x > x {
                x += 1;
                ' '.fmt(f)?;
            }

            if point.x == x {
                // Write point
                'O'.fmt(f)?;
                x += 1;
            }
        }

        '\n'.fmt(f)?;

        for _ in 0..=self.width() {
            '-'.fmt(f)?;
        }

        Ok(())
    }
}

impl Points {
    fn advance(&mut self) {
        self.0.iter_mut().for_each(|point| point.advance());
        self.0.sort();
    }

    fn height(&self) -> i64 {
        let all_ys = self.0.iter().map(|point| point.y);

        let min_y = all_ys.clone().min().unwrap();
        let max_y = all_ys.max().unwrap();

        max_y - min_y
    }

    fn width(&self) -> i64 {
        let all_xs = self.0.iter().map(|point| point.x);

        let min_x = all_xs.clone().min().unwrap();
        let max_x = all_xs.max().unwrap();

        max_x - min_x
    }
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let pattern =
        Regex::new(r"position=\s*<\s*(-?\d+),\s*(-?\d+)>\s*velocity=\s*<\s*(-?\d+),\s*(-?\d+)>").unwrap();
    let mut points: Points = pattern
        .captures_iter(input)
        .map(|caps| Point {
            x: caps.parse(1),
            y: caps.parse(2),

            dx: caps.parse(3),
            dy: caps.parse(4),
        })
        .collect();

    eprintln!("{}", points.0.len());

    let mut seconds = 0;
    loop {
        points.advance();
        seconds += 1;

        if points.height() < 20 {
            println!("time: {}\n{}", seconds, points);
            sleep(Duration::from_secs(1));
        }
    }

    "HELLO"
}
