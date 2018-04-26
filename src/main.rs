#![feature(test)]
extern crate failure;
extern crate test;
#[macro_use]
extern crate serde_derive;
extern crate csv;

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

#[derive(Debug, Serialize, Default, Clone)]
struct Row {
    timestamp: u64,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
}

impl<'a> From<&'a Line> for Row {
    fn from(l: &Line) -> Row {
        let mut res = Row::default();
        for ins in &l.instructions {
            match ins {
                &Instruction::X(val) => res.x = Some(val),
                &Instruction::Y(val) => res.y = Some(val),
                &Instruction::Z(val) => res.z = Some(val),
                _ => {}
            }
        }
        res
    }
}

impl From<Line> for Row {
    fn from(l: Line) -> Row {
        From::from(&l)
    }
}

fn compute_csv(program: Program) -> Vec<Row> {
    let mut current: Row = From::from(&program.lines[0]);
    let mut out = Vec::new();
    for line in &program.lines {
        let mut row: Row = From::from(line);
        if let Some(xval) = &row.x {
            current.x = Some(*xval);
        }
        if let Some(yval) = &row.y {
            current.y = Some(*yval);
        }
        if let Some(zval) = &row.z {
            current.z = Some(*zval);
        }
        out.push(current.clone());
    }
    out
}

fn write_csv<P: AsRef<Path>>(filename: P, csv_data: &[Row]) {
    use std::fs::File;

    let f = File::create(filename).unwrap();
    let mut writer = csv::Writer::from_writer(f);

    for row in csv_data {
        writer.serialize(row).unwrap();
    }
}

fn main() {
    let program = parse_file("data/raw_gcode.NC");
    let csv_data = compute_csv(program);
    write_csv("results.csv", &csv_data);
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
