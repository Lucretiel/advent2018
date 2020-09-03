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

impl Add<(i32, i32)> for Coords {
    type Output = Coords;

    fn add(self, rhs: (i32, i32)) -> Coords {
        Coords {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Region {
    open_set: HashSet<Coords>,
    closed_set: HashSet<Coords>,
}

impl Region {
    fn new(seed: Coords) -> Self {
        let mut region = Self::default();
        region.open_set.insert(seed);
        region
    }

    fn grow(&self) -> HashSet<Coords> {
        self.open_set
            .iter()
            .cloned()
            .flat_map(|coords| {
                vec![
                    coords + (-1, 0),
                    coords + (0, -1),
                    coords + (1, 0),
                    coords + (0, 1),
                ]
            })
            .filter(|coords| !self.open_set.contains(coords) && !self.closed_set.contains(coords))
            .collect()
    }

    fn apply<I: IntoIterator<Item = Coords>>(&mut self, updates: I) {
        self.closed_set
            .extend(replace(&mut self.open_set, updates.into_iter().collect()));
    }

    fn sealed_size(&self) -> Option<usize> {
        if self.open_set.is_empty() {
            Some(self.closed_set.len())
        } else {
            None
        }
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

    let mut occupied: HashMap<Coords, usize> = HashMap::from_iter(coords.iter().cloned().map(|coord| (coord, 1)));
    let mut regions: Vec<Region> = coords.into_iter().map(Region::new).collect();
    let mut proposed_growths: Vec<HashSet<Coords>> = Vec::with_capacity(regions.len());

    eprintln!("{}", regions.len());

    for _ in 0..1000 {
        proposed_growths.clear();

        proposed_growths.extend(regions.iter().map(|region| region.grow()));
        proposed_growths.iter().flat_map(|growth| growth.iter()).for_each(|cell| {
            occupied.entry(*cell)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        });

        regions.iter_mut().zip(proposed_growths.iter()).for_each(|(region, growth)| {
            region.apply(growth.iter().filter(|cell| occupied[cell] == 1).cloned())
        });
    }

    regions.iter().filter_map(|region| region.sealed_size()).max().unwrap()
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
