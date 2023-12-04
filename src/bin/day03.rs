#[allow(unused)]
use std::collections::BTreeSet;
use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let board: Board = input.lines().collect();
    Ok(board
        .part_numbers()
        .map(|n| n.as_str.parse::<u32>().expect("number parsing failed"))
        .sum())
}

fn part_two(input: &str) -> Result<u32> {
    let board: Board = input.lines().collect();
    Ok(board.gears().map(|g| g.gear_ratio()).sum())
}

#[derive(Debug)]
struct Board<'i> {
    lines: usize,
    cols: usize,
    numbers: Vec<Number<'i>>,
    symbols: HashSet<Symbol>,
}

impl<'i> Board<'i> {
    fn part_numbers(&'i self) -> impl Iterator<Item = &'i Number<'i>> {
        self.numbers.iter().filter(|n| {
            self.adjacent_positions(n)
                .any(|pos| self.has_symbol_at(pos))
        })
    }

    fn gears(&self) -> impl Iterator<Item = Gear<'_>> {
        let mut rev_lookup: BTreeMap<(usize, usize), Vec<&Number<'_>>> = BTreeMap::new();
        for pn in self.part_numbers() {
            for pos in self.adjacent_positions(pn) {
                rev_lookup
                    .entry(pos)
                    .and_modify(|ns| ns.push(pn))
                    .or_insert(vec![pn]);
            }
        }

        rev_lookup.retain(|_, pns| pns.len() == 2);

        self.symbols
            .iter()
            .filter(|s| s.ch == '*')
            .filter_map(move |s| {
                let pos = (s.line_no, s.col_no);
                let part_numbers = rev_lookup.remove(&pos)?;
                if part_numbers.len() == 2 {
                    Some(Gear {
                        line_no: s.line_no,
                        col_no: s.col_no,
                        part_numbers,
                    })
                } else {
                    None
                }
            })
    }

    fn adjacent_positions(&self, n: &Number<'_>) -> impl Iterator<Item = (usize, usize)> {
        let mut pos_set = BTreeSet::new();
        for line_idx in n.line_no.saturating_sub(1)..=n.line_no.saturating_add(1) {
            if line_idx >= self.lines {
                continue;
            }
            for col_idx in n.col_no.saturating_sub(1)..=n.col_no.saturating_add(n.as_str.len()) {
                if col_idx >= self.cols
                    || (line_idx == n.line_no
                        && (n.col_no..n.col_no + n.as_str.len()).contains(&col_idx))
                {
                    continue;
                }
                pos_set.insert((line_idx, col_idx));
            }
        }
        pos_set.into_iter()
    }

    fn has_symbol_at(&self, (line_no, col_no): (usize, usize)) -> bool {
        if line_no >= self.lines || col_no >= self.cols {
            return false;
        }
        self.symbols.contains(&Symbol {
            // ch is not used in the check
            ch: '\0',
            line_no,
            col_no,
        })
        // self.symbols
        //     .iter()
        //     .any(|s| s.line_no == line_no && s.col_no == col_no)
    }
}

impl<'i> std::iter::FromIterator<&'i str> for Board<'i> {
    fn from_iter<T: IntoIterator<Item = &'i str>>(iter: T) -> Self {
        let mut numbers = Vec::new();
        let mut symbols = HashSet::new();
        let mut lines = 0;
        let mut cols = 0;
        for (line_no, line) in iter.into_iter().enumerate() {
            lines += 1;
            if line_no == 0 {
                cols = line.len();
            } else {
                assert!(cols == line.len());
            }

            let mut start_idx = None;
            let mut push_num = |start_idx, end_idx| {
                if let Some(start_idx) = start_idx {
                    numbers.push(Number {
                        as_str: &line[start_idx..end_idx],
                        line_no,
                        col_no: start_idx,
                    });
                }
            };
            for (col_no, ch) in line.char_indices() {
                match ch {
                    '0'..='9' => {
                        start_idx.get_or_insert(col_no);
                    }
                    '.' => {
                        push_num(start_idx.take(), col_no);
                    }
                    _sym => {
                        push_num(start_idx.take(), col_no);
                        symbols.insert(Symbol {
                            ch,
                            line_no,
                            col_no,
                        });
                    }
                };
            }
            push_num(start_idx.take(), line.len());
        }
        Self {
            lines,
            cols,
            numbers,
            symbols,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Number<'i> {
    as_str: &'i str,
    line_no: usize,
    col_no: usize,
}

#[derive(Debug, Eq)]
struct Symbol {
    ch: char,
    line_no: usize,
    col_no: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Gear<'i> {
    part_numbers: Vec<&'i Number<'i>>,
    line_no: usize,
    col_no: usize,
}

impl<'i> Gear<'i> {
    fn gear_ratio(&self) -> u32 {
        self.part_numbers
            .iter()
            .map(|pn| {
                pn.as_str
                    .parse::<u32>()
                    .expect("part number parsing failed")
            })
            .product()
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        // the character does not participate in equality checks
        // self.ch == other.ch &&
        self.line_no == other.line_no && self.col_no == other.col_no
    }
}

impl std::hash::Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // the character does not participate in hashing
        // self.ch.hash(state);
        self.line_no.hash(state);
        self.col_no.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! symbol {
        ($ch:literal, $l:literal, $c:literal) => {
            Symbol {
                ch: $ch,
                line_no: $l,
                col_no: $c,
            }
        };
    }

    macro_rules! number {
        ($num:literal, $l:literal, $c:literal) => {
            Number {
                as_str: $num,
                line_no: $l,
                col_no: $c,
            }
        };
    }
    const INPUT: &str = r#"467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598.."#;

    #[test]
    fn test_board_parse_numbers() {
        let b: Board = INPUT.lines().collect();
        assert_eq!(b.numbers.len(), 10);
        let mut numbers = b.numbers.into_iter();
        assert_eq!(numbers.next(), Some(number!("467", 0, 0)));
        assert_eq!(numbers.last(), Some(number!("598", 9, 5)));
    }

    #[test]
    fn test_board_parse_symbols() {
        let b: Board = INPUT.lines().collect();
        assert_eq!(b.symbols.len(), 6);
        assert!(b.symbols.contains(&symbol!('*', 1, 3)));
        assert!(b.symbols.contains(&symbol!('*', 8, 5)));
    }

    #[test]
    fn test_part_numbers() {
        let b: Board = INPUT.lines().collect();
        let mut part_numbers: Vec<_> = b.part_numbers().collect();
        let mut expected = vec![
            &number!("467", 0, 0),
            &number!("35", 2, 2),
            &number!("633", 2, 6),
            &number!("617", 4, 0),
            &number!("592", 6, 2),
            &number!("755", 7, 6),
            &number!("664", 9, 1),
            &number!("598", 9, 5),
        ];
        part_numbers.sort_by_key(|n| n.as_str.parse::<u32>().unwrap());
        expected.sort_by_key(|n| n.as_str.parse::<u32>().unwrap());
        assert_eq!(part_numbers, expected);
    }

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 4361);
        Ok(())
    }

    #[test]
    fn test_gears() {
        let b: Board = INPUT.lines().collect();
        let expected = vec![
            Gear {
                line_no: 1,
                col_no: 3,
                part_numbers: vec![&number!("467", 0, 0), &number!("35", 2, 2)],
            },
            Gear {
                line_no: 8,
                col_no: 5,
                part_numbers: vec![&number!("755", 7, 6), &number!("598", 9, 5)],
            },
        ];
        let mut actual = b.gears().collect::<Vec<_>>();
        actual.sort_by_key(|g| (g.line_no, g.col_no));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_gear_ratio() {
        let b: Board = INPUT.lines().collect();
        let mut gears: Vec<_> = b.gears().map(|g| g.gear_ratio()).collect();
        gears.sort();
        assert_eq!(gears, vec![16345, 451490])
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 467835);
        Ok(())
    }
}
