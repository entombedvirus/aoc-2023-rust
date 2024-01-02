use std::collections::{BTreeMap, VecDeque};

use anyhow::{Context, Result};
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let mut p = Puzzle::parse(input)?;
    p.compute_min_steps();
    Ok(p.num_reachable_tiles(64))
}

fn part_two(input: &str) -> Result<usize> {
    let mut p = Puzzle::parse(input)?;
    p.compute_min_steps();
    Ok(p.compute_reachable_tiles(26501365))
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
    start_pos: (isize, isize),
    min_steps_needed: BTreeMap<(isize, isize), u32>,

    infinite_tiles: bool,
}

impl Puzzle {
    fn parse(input: &str) -> Result<Self> {
        let num_rows = input.lines().count();
        let num_cols = input
            .lines()
            .next()
            .map(|line| line.len())
            .context("input is empty")?;
        let tiles: Vec<_> = input
            .lines()
            .flat_map(|line| line.as_bytes())
            .map(|ch| ch.into())
            .collect();
        let start_pos = {
            let idx = tiles
                .iter()
                .position(|t| t == &Tile::Start)
                .expect("start tile is missing");
            ((idx / num_cols) as isize, (idx % num_cols) as isize)
        };
        Ok(Self {
            num_rows,
            num_cols,
            start_pos,
            tiles,
            infinite_tiles: false,
            min_steps_needed: Default::default(),
        })
    }

    fn get(&self, mut row: isize, mut col: isize) -> Option<Tile> {
        // normalize out of bound coordinates to simulate infinite tiles
        if self.infinite_tiles {
            row %= self.num_rows as isize;
            col %= self.num_cols as isize;
            if row < 0 {
                row = self.num_rows as isize + row;
            }
            if col < 0 {
                col = self.num_cols as isize + col
            }
        }
        if row < 0 || row >= self.num_rows as isize || col < 0 || col >= self.num_cols as isize {
            None
        } else {
            let idx = row * self.num_cols as isize + col;
            self.tiles.get(idx as usize).copied()
        }
    }

    // See: https://github.com/villuna/aoc23/wiki/A-Geometric-solution-to-advent-of-code-2023,-day-21
    fn compute_min_steps(&mut self) {
        let start_pos = self.start_pos;
        let mut queue = VecDeque::new();
        queue.push_back((start_pos, 0));

        if self.min_steps_needed.is_empty() {
            // BFS to find the shortest path to each tile
            while let Some((pos @ (row, col), min_steps)) = queue.pop_front() {
                match self.get(row, col) {
                    Some(Tile::Grass | Tile::Start) => {
                        self.min_steps_needed.entry(pos).or_insert(min_steps);
                        for (dr, dc) in [(0, 1), (0, -1), (-1, 0), (1, 0)] {
                            let new_pos = (row + dr, col + dc);
                            if let Some(Tile::Start | Tile::Grass) = self.get(new_pos.0, new_pos.1)
                            {
                                self.min_steps_needed.entry(new_pos).or_insert_with(|| {
                                    queue.push_back((new_pos, min_steps + 1));
                                    min_steps + 1
                                });
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn num_reachable_tiles(&self, num_steps: u32) -> usize {
        let parity = |n| n % 2;
        // find all the squares reachable within num_steps
        self.min_steps_needed
            .values()
            .filter(|steps| **steps <= num_steps && parity(**steps) == parity(num_steps))
            .count()
    }

    fn compute_reachable_tiles(&self, num_steps: u32) -> usize {
        assert!(
            num_steps as usize >= self.num_cols,
            "use num_reachable_tiles instead"
        );
        // the row and column that the starting tile is on is unbostructed: there are no stones
        // This means that if we can take num_steps straight in up, down, left or right and count how
        // many times the original map repeats.
        let steps_to_edge = num_steps % self.num_cols as u32;

        let even_corners = self
            .min_steps_needed
            .values()
            .filter(|v| **v % 2 == 0 && **v > steps_to_edge)
            .count();
        let odd_corners = self
            .min_steps_needed
            .values()
            .filter(|v| **v % 2 == 1 && **v > steps_to_edge)
            .count();

        let even_full = self
            .min_steps_needed
            .values()
            .filter(|v| **v % 2 == 0)
            .count();
        let odd_full = self
            .min_steps_needed
            .values()
            .filter(|v| **v % 2 == 1)
            .count();

        let n = ((num_steps as usize) - (self.num_cols as usize / 2)) / self.num_cols as usize;

        ((n + 1) * (n + 1)) * odd_full + (n * n) * even_full - (n + 1) * odd_corners
            + n * even_corners
    }
}

#[allow(unused)]
#[derive(Debug)]
struct TilePrinter<'i>(&'i Puzzle);

impl std::fmt::Display for TilePrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let puzzle = self.0;
        let min_steps_needed = &puzzle.min_steps_needed;
        for row in 0..puzzle.num_rows {
            for col in 0..puzzle.num_cols {
                let pos = (row as isize, col as isize);
                if min_steps_needed.contains_key(&pos) {
                    write!(f, "{}", min_steps_needed[&pos])?;
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
        let mut p = Puzzle::parse(INPUT)?;
        p.compute_min_steps();
        let ks = [(1, 2), (2, 4), (3, 6), (6, 16)];
        for (num_steps, expected) in ks {
            let n = p.num_reachable_tiles(num_steps);
            assert_eq!(n, expected, "num_steps: {num_steps}");
        }
        Ok(())
    }
}
