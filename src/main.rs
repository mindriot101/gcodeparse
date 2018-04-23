#![feature(test)]
extern crate failure;
extern crate test;

use failure::{Error, ResultExt};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub enum Instruction {
    G(u32),
    T(u32),
    M(u32),
    S(u32),
    X(f32),
    Y(f32),
    Z(f32),
    I(f32),
    J(f32),
    K(f32),
    F(f32),
    R(f32),
}

impl Instruction {
    fn from_pair(typ: &str, value: &str) -> Option<Instruction> {
        match typ {
            "G" => Some(Instruction::G(value.parse().unwrap())),
            "T" => Some(Instruction::T(value.parse().unwrap())),
            "M" => Some(Instruction::M(value.parse().unwrap())),
            "S" => Some(Instruction::S(value.parse().unwrap())),
            "X" => Some(Instruction::X(value.parse().unwrap())),
            "Y" => Some(Instruction::Y(value.parse().unwrap())),
            "Z" => Some(Instruction::Z(value.parse().unwrap())),
            "I" => Some(Instruction::I(value.parse().unwrap())),
            "J" => Some(Instruction::J(value.parse().unwrap())),
            "K" => Some(Instruction::K(value.parse().unwrap())),
            "F" => Some(Instruction::F(value.parse().unwrap())),
            "R" => Some(Instruction::F(value.parse().unwrap())),
            _ => {
                eprintln!("UNKNOWN CODE: {} => {}", typ, value);
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct Line {
    pub instructions: Vec<Instruction>,
    pub number: Option<u32>,
}

impl Line {
    pub fn new() -> Self {
        Line {
            instructions: Vec::new(),
            number: None,
        }
    }
}

fn pairwise<T>(t: &[T]) -> impl Iterator<Item = (&T, &T)>
where
    T: std::fmt::Debug,
{
    assert!(t.len() % 2 == 0, "{:?}", t);
    PairwiseIterator { arr: t, counter: 0 }
}

struct PairwiseIterator<'a, T: 'a> {
    arr: &'a [T],
    counter: usize,
}

impl<'a, T: 'a> Iterator for PairwiseIterator<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter >= self.arr.len() {
            return None;
        }

        let result = Some((&self.arr[self.counter], &self.arr[self.counter + 1]));
        self.counter += 2;
        result
    }
}

impl FromStr for Line {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Vec::new();
        let mut last = 0;
        for (index, matched) in s.match_indices(|c: char| c.is_alphabetic()) {
            if last != index {
                result.push(&s[last..index]);
            }

            result.push(matched);
            last = index + matched.len();
        }

        result.push(&s[last..]);

        let mut out = Line::new();
        for (ins_type, value) in pairwise(&result) {
            match ins_type {
                &"N" => out.number = Some(value.parse().unwrap()),
                _ => match Instruction::from_pair(ins_type, value) {
                    Some(i) => out.instructions.push(i),
                    None => {}
                },
            }
        }

        Ok(out)
    }
}

#[derive(Debug)]
pub struct Program {
    pub lines: Vec<Line>,
}

impl Program {
    pub fn new() -> Self {
        Program { lines: Vec::new() }
    }
}

pub fn parse_string(s: &str) -> Program {
    let mut program = Program::new();
    for line in s.lines() {
        let mut text = line;

        /* Strip comments */
        let text = if let Some(idx) = text.find('(') {
            &text[..idx]
        } else {
            &text
        };

        if text.is_empty() {
            continue;
        }

        assert!(!text.contains('('));

        let line = Line::from_str(&text).context("parsing line").unwrap();
        program.lines.push(line);
    }

    program
}

fn parse_file<P: AsRef<Path>>(path: P) -> Program {
    let mut f = File::open(path).expect("opening file");
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    parse_string(&s)
}

fn main() {
    let program = parse_file("data/raw_gcode.NC");
    println!("{:#?}", program);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &'static str = include_str!("../data/raw_gcode.NC");

    #[bench]
    fn bench_program(b: &mut test::Bencher) {
        b.iter(|| {
            let _ = parse_string(SRC);
        });
    }
}
