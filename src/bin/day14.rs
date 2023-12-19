#![allow(unused, dead_code)]

use anyhow::Result;
use aoc::{must_parse, runner, wait};
use nom::{
    bytes::complete::is_a, character::complete::newline, combinator::map, multi::separated_list1,
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let p = Puzzle::parse(input)?;
    Ok((0..p.num_cols())
        .map(|col_num| p.column_score(col_num))
        .sum())
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Puzzle<'i> {
    rows: Vec<&'i str>,
}

impl<'i> Puzzle<'i> {
    fn parse(input: &'i str) -> Result<Self> {
        let parse_lines = separated_list1(newline, is_a("O.#"));
        let parser = map(parse_lines, |rows| Self { rows });
        must_parse(parser, input)
    }

    fn num_cols(&self) -> usize {
        self.rows.first().map(|r| r.len()).unwrap_or(0)
    }

    fn column_score(&self, col_num: usize) -> u32 {
        let mut acc = 0;
        let mut score = self.rows.len() as u32;
        for (row_idx, ch) in self.col_iter(col_num).enumerate() {
            match ch {
                '.' => continue,
                'O' => {
                    acc += score;
                    score -= 1;
                }
                '#' => {
                    score = (self.rows.len() - row_idx - 1) as u32;
                }
                unknown => unreachable!("unknown char: {unknown}"),
            };
        }
        acc
    }

    fn col_iter(&'i self, col_num: usize) -> impl Iterator<Item = char> + 'i {
        self.rows.iter().map(move |r| r.as_bytes()[col_num].into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#...."#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 136);
        Ok(())
    }
}
