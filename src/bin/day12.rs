#![allow(unused, dead_code)]

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
};

use anyhow::Result;
use aoc::{must_parse, runner, wait};
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
        // .map(|line| line.possibilities().count())
        .map(|line| line.num_possibilities())
        .sum())
}

fn part_two(input: &str) -> Result<usize> {
    let puzzle = Puzzle::parse(input)?;
    Ok(puzzle
        .lines
        .iter()
        .map(|line| line.unfold().num_possibilities())
        .sum())
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

    fn num_possibilities(&self) -> usize {
        let cache = RefCell::new(HashMap::new());
        count_num_possibilities(&self.springs, &self.groups, &cache)
    }

    fn unfold(&self) -> Self {
        let springs = [self.springs.as_str()].repeat(5).join("?");
        let groups = self.groups.repeat(5);
        Self { springs, groups }
    }
}

fn count_num_possibilities<'a>(
    springs: &str,
    groups: &'a [u8],
    cache: &RefCell<HashMap<(String, &'a [u8]), usize>>,
) -> usize {
    let springs = springs.trim_matches('.');

    // base cases that do not need recursion
    if springs.is_empty() && groups.is_empty() {
        return 1;
    } else if groups.is_empty() {
        // dots or questions only are okay
        return if !springs.contains('#') { 1 } else { 0 };
    } else if !springs.contains('?') {
        return if PuzzleLine::is_valid(springs, groups) {
            1
        } else {
            0
        };
    }

    let (first, rest) = springs.split_once('.').unwrap_or((springs, ""));
    match first.find('?') {
        None => {
            let (gfirst, grest) = groups.split_at(1);
            if PuzzleLine::is_valid(first, gfirst) {
                count_num_possibilities(rest, grest, cache)
            } else {
                0
            }
        }
        Some(idx) => {
            let key = (springs.to_owned(), groups);
            let cache_value = cache.borrow().get(&key).copied();
            if let Some(v) = cache_value {
                v
            } else {
                let mut replaced = springs.to_string();
                replaced.replace_range(idx..idx + 1, "#");
                let c1 = count_num_possibilities(&replaced, groups, cache);
                replaced.replace_range(idx..idx + 1, ".");
                let c2 = count_num_possibilities(&replaced, groups, cache);
                cache.borrow_mut().insert(key, c1 + c2);
                c1 + c2
            }
        }
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
