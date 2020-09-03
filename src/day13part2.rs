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
use lazy_format::lazy_format;

use gridly::prelude::*;
use gridly_basic_grid::*;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Track {
    Empty,
    Vertical,
    Horizontal,
    RightBend,
    LeftBend,
    Intersection,
}

impl Track {
    fn as_char(self) -> char {
        match self {
            Track::Empty => ' ',
            Track::Vertical => '|',
            Track::Horizontal => '–',
            Track::RightBend => '/',
            Track::LeftBend => '\\',
            Track::Intersection => '+',
        }
    }
}

impl Default for Track {
    fn default() -> Self {
        Track::Empty
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Turn {
    Left,
    Straight,
    Right,
}

impl Turn {
    fn next(self) -> Self {
        match self {
            Turn::Left => Turn::Straight,
            Turn::Straight => Turn::Right,
            Turn::Right => Turn::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Cart {
    direction: Direction,
    next_turn: Turn,
}

impl Cart {
    fn new(direction: Direction) -> Self {
        Self {
            direction,
            next_turn: Turn::Left,
        }
    }
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let lines: Vec<&str> = input.lines().map(|line| line.trim_end()).collect();
    let num_rows = Rows(lines.len() as isize);
    let num_columns = Columns(lines.iter().map(|line| line.len()).max().unwrap() as isize);

    let mut grid: VecGrid<Track> = VecGrid::new((num_rows, num_columns)).unwrap();
    let mut carts: HashMap<Location, Cart> = HashMap::new();

    for (line, row) in lines.iter().zip(RowRange::span(0.into(), num_rows)) {
        for (c, column) in line.chars().zip(ColumnRange::span(0.into(), num_columns)) {
            let location = row + column;

            let track = match c {
                ' ' => Track::Empty,
                '+' => Track::Intersection,
                '|' => Track::Vertical,
                '-' => Track::Horizontal,
                '/' => Track::RightBend,
                '\\' => Track::LeftBend,

                '^' => {
                    carts.insert(location, Cart::new(Up));
                    Track::Vertical
                }
                'v' | 'V' => {
                    carts.insert(location, Cart::new(Down));
                    Track::Vertical
                }
                '>' => {
                    carts.insert(location, Cart::new(Right));
                    Track::Horizontal
                }
                '<' => {
                    carts.insert(location, Cart::new(Left));
                    Track::Horizontal
                }
                c => panic!("Unexpected character: {}", c),
            };

            grid.set(location, track).unwrap()
        }
    }

    let grid = grid;

    let mut ordered_carts: Vec<(Location, Cart)> = Vec::with_capacity(carts.len());
    let mut removed: HashSet<Location> = HashSet::with_capacity(carts.len());
    loop {
        if carts.len() == 1 {
            let location = *carts.keys().next().unwrap();
            break lazy_format!("X: {}, Y: {}", location.column.0, location.row.0);
        }

        // Collect the carts in sorted order
        ordered_carts.clear();
        ordered_carts.extend(carts.iter().map(|(loc, cart)| (*loc, *cart)));
        ordered_carts.sort_by_key(|(loc, _)| loc.row_ordered());

        removed.clear();

        for (location, mut cart) in ordered_carts.iter().cloned() {
            // This cart was removed in a collision; skip it
            if removed.contains(&location) {
                continue;
            }

            // Move the cart
            carts.remove(&location);
            let new_location = location.step(cart.direction);

            // Check for collisions
            if carts.remove(&new_location).is_some() {
                removed.insert(new_location);
                continue;
            }

            // Re-orient the cart
            match grid.get(new_location).unwrap() {
                Track::Empty => panic!("Cart {:?} entered empty track at {:?} from {:?}", cart, new_location, location),
                Track::Horizontal | Track::Vertical => {},
                Track::LeftBend => match cart.direction {
                    Up | Down => { cart.direction = cart.direction.counter_clockwise(); }
                    Left | Right => { cart.direction = cart.direction.clockwise(); }
                }
                Track::RightBend => match cart.direction {
                    Up | Down => { cart.direction = cart.direction.clockwise(); }
                    Left | Right => { cart.direction = cart.direction.counter_clockwise(); }
                }
                Track::Intersection => {
                    match cart.next_turn {
                        Turn::Straight => {},
                        Turn::Left => { cart.direction = cart.direction.counter_clockwise(); }
                        Turn::Right => { cart.direction = cart.direction.clockwise(); }
                    }
                    cart.next_turn = cart.next_turn.next();
                }
            }

            // Change the cart in the carts table
            carts.insert(new_location, cart);
        }
    }
}
