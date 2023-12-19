#![feature(iter_map_windows)]

use anyhow::{Context, Result};
use aoc::{must_parse, runner};
use nom::{
    bytes::complete::is_a,
    character::complete::newline,
    combinator::map,
    multi::{count, separated_list1},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let puzzles = Puzzle::parse(input)?;
    Ok(puzzles
        .into_iter()
        .map(|p| p.reflection_score(Reflection::check))
        .sum())
}

fn part_two(input: &str) -> Result<usize> {
    let puzzles = Puzzle::parse(input)?;
    Ok(puzzles
        .into_iter()
        .map(|p| p.reflection_score(Reflection::check_smudged))
        .sum())
}

#[derive(Debug)]
struct Puzzle {
    rows: Vec<u64>,
    // rows transposed as columns
    cols: Vec<u64>,
}

impl Puzzle {
    fn parse(input: &str) -> Result<Vec<Self>> {
        fn to_number<T: AsRef<str>>(as_str: T) -> u64 {
            assert!(as_str.as_ref().len() <= u64::BITS as usize);
            // convert the string to a number by mapping each # to 2^(idx)
            // ex:    |#.#..#| -> 2^5 + 2^3 + 2^0 = 41
            // index: |543210|
            as_str
                .as_ref()
                .chars()
                .rev()
                .enumerate()
                .filter_map(|(idx, ch)| (ch == '#').then_some(1_u64 << idx))
                .sum()
        }
        fn transpose(rows: Vec<&str>) -> Vec<String> {
            let mut cols: Vec<String> = vec![String::new(); rows.first().map_or(0, |r| r.len())];
            for row in rows {
                for (idx, ch) in row.char_indices() {
                    cols[idx].push(ch);
                }
            }
            cols
        }

        let parse_puzzle = map(separated_list1(newline, is_a("#.")), |lines: Vec<&str>| {
            Self {
                rows: lines.iter().map(to_number).collect(),
                cols: transpose(lines).iter().map(to_number).collect(),
            }
        });
        let parser = separated_list1(count(newline, 2), parse_puzzle);
        must_parse(parser, input)
    }

    fn reflection_score(&self, pred: impl FnMut(usize, usize, &[u64]) -> bool) -> usize {
        use Reflection::*;
        match self.reflection(pred) {
            Horizontal(row_num) => 100 * row_num,
            Vertical(col_num) => col_num,
        }
    }

    fn reflection(&self, mut pred: impl FnMut(usize, usize, &[u64]) -> bool) -> Reflection {
        use Reflection::*;
        Self::find_reflection(&self.rows, &mut pred)
            .map(Horizontal)
            .or_else(|| Self::find_reflection(&self.cols, pred).map(Vertical))
            .with_context(|| format!("no reflection line found for puzzle: {:?}", self))
            .unwrap()
    }

    fn find_reflection(
        lines: &[u64],
        mut pred: impl FnMut(usize, usize, &[u64]) -> bool,
    ) -> Option<usize> {
        (0..lines.len())
            .map_windows(|&[r1, r2]| {
                // r2 number of rows above the reflection line or number cols depending on
                // whether lines is rows or cols
                pred(r1, r2, lines).then_some(r2)
            })
            .find_map(|row| row)
    }
}

#[derive(Debug)]
enum Reflection {
    Horizontal(usize),
    Vertical(usize),
}

impl Reflection {
    fn check(r1: usize, r2: usize, lines: &[u64]) -> bool {
        if lines[r1] != lines[r2] {
            return false;
        }
        if r1 > 0 && r2 < lines.len().saturating_sub(1) {
            Self::check(r1 - 1, r2 + 1, lines)
        } else {
            true
        }
    }

    fn check_smudged(mut r1: usize, mut r2: usize, lines: &[u64]) -> bool {
        let mut found = false;
        let valid_range = 0..lines.len();
        while valid_range.contains(&r1) && valid_range.contains(&r2) {
            if lines[r1] != lines[r2] {
                if found {
                    found = false;
                    break;
                }

                // xor computes which corresponding bit positions are different
                let mut diff_bit_positions = lines[r1] as i64 ^ lines[r2] as i64;
                // x & (x - 1) clears the least significant bit that is set. If there was only
                // one bit that was set, this will return zero.
                diff_bit_positions &= diff_bit_positions - 1;
                if diff_bit_positions == 0 {
                    found = true;
                } else {
                    found = false;
                    break;
                }
            }
            if let Some(nr1) = r1.checked_sub(1) {
                r1 = nr1;
                r2 += 1;
            } else {
                break;
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#"#;

    #[test]
    fn test_parsing() -> Result<()> {
        let zs = Puzzle::parse(INPUT)?;
        assert_eq!(zs.len(), 2);
        assert_eq!(zs[0].rows.len(), 7);
        assert_eq!(zs[0].cols.len(), 9);
        assert_eq!(zs[1].rows.len(), 7);
        assert_eq!(zs[1].cols.len(), 9);
        Ok(())
    }

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 405);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 400);
        Ok(())
    }
}
