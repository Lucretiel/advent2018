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


#[inline(always)]
fn solve(input: &str) -> impl Display {
    let pattern = Regex::new(r"([A-Z]) must be finished before step ([A-Z])").unwrap();

    // Mapping of step -> prereqs
    let mut steps: HashMap<char, HashSet<char>> = HashMap::new();

    pattern.captures_iter(&input).for_each(|cap| {
        let prereq = cap.parse(1);
        let step = cap.parse(2);

        steps.entry(prereq).or_default();
        steps.entry(step).or_default().insert(prereq);
    });

    let mut result = String::new();

    loop {
        let step = steps.iter()
            .filter(|(_step, prereqs)| prereqs.is_empty())
            .min_by_key(|(step, _prereqs)| *step);

        match step {
            None => break result,
            Some((&step, _)) => {
                result.push(step);
                steps.remove(&step);
                steps.values_mut().for_each(|prereqs| {prereqs.remove(&step);});
            }
        }

    }
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
