#![allow(unused, dead_code)]

use std::collections::{
    btree_map::{Entry, OccupiedEntry},
    BTreeMap, BTreeSet,
};

use anyhow::{Context, Result};
use aoc::{runner, wait};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let p = Puzzle::parse(input)?;
    let start_pos = {
        let idx = p
            .tiles
            .iter()
            .position(|t| t == &Tile::Start)
            .expect("start tile is missing");
        ((idx / p.num_cols) as isize, (idx % p.num_cols) as isize)
    };
    let mut cache = BTreeMap::new();
    p.num_reachable_tiles2(start_pos, 64, &mut cache);
    cache
        .get(&(start_pos, 64))
        .map(|positions| positions.len() as u32)
        .context("no result")
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Grass = b'.' as isize,
    Stone = b'#' as isize,
    Start = b'S' as isize,
}

impl std::convert::From<&u8> for Tile {
    fn from(ch: &u8) -> Self {
        match ch {
            b'.' => Self::Grass,
            b'#' => Self::Stone,
            b'S' => Self::Start,
            _ => panic!("invalid tile"),
        }
    }
}

#[derive(Debug)]
struct Puzzle {
    num_rows: usize,
    num_cols: usize,
    tiles: Vec<Tile>,
}

type PositionSet = BTreeSet<(isize, isize)>;
// row, col, num_steps
type PositionKey = ((isize, isize), u32);

impl Puzzle {
    fn parse(input: &str) -> Result<Self> {
        let num_rows = input.lines().count();
        let num_cols = input
            .lines()
            .next()
            .map(|line| line.len())
            .context("input is empty")?;
        Ok(Self {
            num_rows,
            num_cols,
            tiles: input
                .lines()
                .flat_map(|line| line.as_bytes())
                .map(|ch| ch.into())
                .collect(),
        })
    }

    fn get(&self, row: isize, col: isize) -> Option<Tile> {
        let idx = row * self.num_cols as isize + col;
        self.tiles.get(idx as usize).copied()
    }

    fn num_reachable_tiles2(
        &self,
        start: (isize, isize),
        num_steps: u32,
        cache: &mut BTreeMap<PositionKey, PositionSet>,
    ) {
        let key = (start, num_steps);
        if cache.contains_key(&key) {
            // noop
        } else if num_steps == 0 {
            let mut s = BTreeSet::new();
            s.insert(start);
            cache.insert(key, s);
        } else {
            let neighbors: Vec<_> = [(0, 1), (0, -1), (-1, 0), (1, 0)]
                .into_iter()
                .map(|(dr, dc)| (start.0 + dr, start.1 + dc))
                .collect();

            let mut merged = BTreeSet::new();
            for new_pos in neighbors {
                if let Some(Tile::Grass | Tile::Start) = self.get(new_pos.0, new_pos.1) {
                    self.num_reachable_tiles2(new_pos, num_steps - 1, cache);
                    let key = (new_pos, num_steps - 1);
                    if let Some(result) = cache.get(&key) {
                        merged.extend(result);
                    }
                }
            }
            cache.insert(key, merged);
        }
    }

    fn num_reachable_tiles(&self, num_steps: u32) -> u32 {
        let start_pos = {
            let idx = self
                .tiles
                .iter()
                .position(|t| t == &Tile::Start)
                .expect("start tile is missing");
            (
                (idx / self.num_cols) as isize,
                (idx % self.num_cols) as isize,
            )
        };
        let mut fringe = vec![(start_pos, num_steps)];
        let mut end_positions = BTreeSet::new();

        while let Some((pos @ (row, col), steps_left)) = fringe.pop() {
            match self.get(row, col) {
                Some(Tile::Grass | Tile::Start) => {
                    if steps_left == 0 {
                        if end_positions.insert(pos) {
                            // let p = TilePrinter(self, &end_positions);
                            // eprintln!("{p}");
                            // wait();
                        }
                    } else {
                        for (dr, dc) in [(0, 1), (0, -1), (-1, 0), (1, 0)] {
                            let new_pos = (row + dr, col + dc);
                            match self.get(new_pos.0, new_pos.1) {
                                Some(Tile::Grass | Tile::Start) => {
                                    fringe.push((new_pos, steps_left - 1))
                                }
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        end_positions.len() as u32
    }
}

#[derive(Debug)]
struct TilePrinter<'i>(&'i Puzzle, &'i BTreeSet<(isize, isize)>);

impl std::fmt::Display for TilePrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(puzzle, end_positions) = self;
        for row in 0..puzzle.num_rows {
            for col in 0..puzzle.num_cols {
                let pos = (row as isize, col as isize);
                if end_positions.contains(&pos) {
                    write!(f, "O")?;
                } else {
                    write!(
                        f,
                        "{}",
                        puzzle.get(row as isize, col as isize).unwrap() as u8 as char
                    )?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
..........."#;

    #[test]
    fn test_part_one() -> Result<()> {
        let p = Puzzle::parse(INPUT)?;
        let start_pos = (5, 5);
        let mut cache = BTreeMap::new();

        let ks = [(1, 2), (2, 4), (3, 6), (6, 16)];
        for (num_steps, expected) in ks {
            p.num_reachable_tiles2(start_pos, num_steps, &mut cache);
            assert_eq!(
                cache.get(&(start_pos, num_steps)).map(BTreeSet::len),
                Some(expected),
                "num_steps: {num_steps}"
            );
        }
        Ok(())
    }
}
