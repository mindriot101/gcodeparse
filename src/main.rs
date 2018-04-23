extern crate failure;

use failure::{Error, ResultExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
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

fn pairwise<T>(t: &[T]) -> impl Iterator<Item = (&T, &T)> {
    assert!(t.len() % 2 == 0);
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

fn main() {
    let f = File::open(
        "/Users/simon/work/MTC/data_munging/data/RAW/NCcode/NIST_MTC_CRADA_PLATE-1.NC",
    ).expect("opening file");
    let reader = BufReader::new(f);

    let mut program = Program::new();

    for line in reader.lines() {
        let text = line.unwrap();

        if !text.starts_with("N") {
            continue;
        }

        let line = Line::from_str(&text).context("parsing line").unwrap();
        program.lines.push(line);
    }
    println!("{:#?}", program);
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;

    #[test]
    fn test_parse_integer() {
        let i = "32";
        assert_eq!(parse_integer(i), IResult::Done(&b""[..], 0, 32));
    }
}
