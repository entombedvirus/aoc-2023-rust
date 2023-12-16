#![allow(unused, dead_code)]

use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{self, newline},
    combinator::{eof, map, opt},
    multi::separated_list1,
    sequence::{preceded, separated_pair, terminated},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let puzzle = Puzzle::parse(input)?;
    Ok(puzzle
        .lines
        .iter()
        .map(|line| line.possibilities().count())
        .sum())
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Puzzle {
    lines: Vec<PuzzleLine>,
}

impl Puzzle {
    fn parse(input: &str) -> Result<Self> {
        let parse_springs = is_not(" ");
        let parse_groups = separated_list1(complete::char(','), complete::u8);
        let parse_line = map(
            separated_pair(parse_springs, complete::char(' '), parse_groups),
            |(springs, groups)| PuzzleLine::new(springs, groups),
        );
        let parse_lines = separated_list1(newline, parse_line);
        let parser = terminated(map(parse_lines, |lines| Self { lines }), opt(newline));
        must_parse(parser, input)
    }
}

#[derive(Debug)]
struct PuzzleLine {
    springs: String,
    groups: Vec<u8>,
}

impl PuzzleLine {
    fn new(springs: &str, groups: Vec<u8>) -> Self {
        let springs = springs.to_owned();
        Self { springs, groups }
    }

    fn is_valid(springs: &str, groups: &[u8]) -> bool {
        let mut pounds = springs.split('.').filter(|grp| grp.is_empty() == false);
        let counts_match = groups.iter().copied().all(|gn| {
            pounds
                .by_ref()
                .next()
                .map(|grp| grp.len() == gn as usize && grp.chars().all(|ch| ch == '#'))
                .unwrap_or(false)
        });
        counts_match && pounds.next().is_none()
    }

    fn possibilities<'a>(&'a self) -> impl Iterator<Item = String> + 'a {
        let num_questions = self
            .springs
            .char_indices()
            .filter(|(_, ch)| *ch == '?')
            .count();
        let num_variations: usize = 1 << num_questions;
        // map i-th bit to i-th question slot
        // with 0 => '.' and 1 => '#'
        (0..num_variations)
            .map(|variation_idx| {
                let mut chars: Vec<_> = self.springs.as_bytes().into();
                let mut qidx = 0;
                for (idx, slot) in chars.iter_mut().enumerate() {
                    if *slot == b'?' {
                        let mask = 1 << qidx;
                        *slot = if variation_idx & mask != 0 {
                            b'#'
                        } else {
                            b'.'
                        };
                        qidx += 1;
                    }
                }
                String::from_utf8(chars).expect("always ascii")
            })
            .filter(|line| Self::is_valid(&line, &self.groups))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 21);
        Ok(())
    }

    #[test]
    fn test_verify() -> Result<()> {
        let p = Puzzle::parse(
            "#.#.### 1,1,3
.#...#....###. 1,1,3
.#.###.#.###### 1,3,1,6
####.#...#... 4,1,1
#....######..#####. 1,6,5
.###.##....# 3,2,1",
        )?;
        for line in p.lines {
            assert!(
                PuzzleLine::is_valid(&line.springs, &line.groups),
                "{line:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn test_possibilities() {
        let line = PuzzleLine::new("???.###", vec![1, 1, 3]);
        let mut iter = line.possibilities();
        assert_eq!(iter.next(), Some(String::from("#.#.###")));
        assert_eq!(iter.next(), None);
    }
}
