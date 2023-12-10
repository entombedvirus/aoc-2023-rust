#![allow(unused, dead_code)]

use anyhow::{Context, Result};
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let maze = Maze::parse(input)?;
    let start = maze.starting_pos();
    let starting_direction = maze
        .valid_directions(start)
        .first()
        .copied()
        .expect("cannot move from starting position");
    let maze_len = maze.navigate(start, starting_direction).count();
    Ok(maze_len / 2)
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Maze {
    rows: usize,
    cols: usize,
    tiles: Vec<Tile>,
}

impl Maze {
    fn parse(input: &str) -> Result<Self> {
        let cols = input
            .lines()
            .next()
            .map(|r| r.len())
            .context("input is empty")?;

        let mut rows = 1;
        let tiles = input
            .lines()
            .flat_map(|r| {
                rows += 1;
                r.chars().map(|ch| ch.try_into())
            })
            .collect::<Result<Vec<Tile>>>()
            .context("tile parsing failed")?;

        Ok(Self { rows, cols, tiles })
    }

    fn starting_pos(&self) -> (usize, usize) {
        let idx = self
            .tiles
            .iter()
            .position(|t| t == &Tile::Start)
            .expect("start position not found in maze");
        let row = idx / self.cols;
        let col = idx % self.cols;
        (row, col)
    }

    fn new_pos(&self, (row, col): (usize, usize), d: Direction) -> Option<(usize, usize)> {
        match d {
            Direction::North => row.checked_sub(1).map(|nr| (nr, col)),
            Direction::South => row.checked_add(1).and_then(|nr| {
                if nr < self.rows {
                    Some((nr, col))
                } else {
                    None
                }
            }),
            Direction::East => col.checked_add(1).and_then(|nc| {
                if nc < self.cols {
                    Some((row, nc))
                } else {
                    None
                }
            }),
            Direction::West => col.checked_sub(1).map(|nc| (row, nc)),
        }
    }

    fn valid_directions(&self, start: (usize, usize)) -> Vec<Direction> {
        use Direction::*;
        let mut dirs = vec![North, East, South, West];
        dirs.retain(|d| {
            self.new_pos(start, *d)
                .map(|new_pos| self.is_connected(start, *d, new_pos))
                .unwrap_or(false)
        });
        dirs
    }

    fn navigate<'a>(
        &'a self,
        start: (usize, usize),
        starting_direction: Direction,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        std::iter::successors(Some((start, starting_direction)), move |(pos, dir)| {
            self.new_pos(*pos, *dir).and_then(|new_pos| {
                if self.is_connected(*pos, *dir, new_pos) && new_pos != start {
                    Some((new_pos, self.new_direction(new_pos, !*dir)))
                } else {
                    None
                }
            })
        })
        .map(|(pos, _)| pos)
    }

    fn new_direction(&self, pos: (usize, usize), old_dir: Direction) -> Direction {
        match self[pos] {
            Tile::Pipe(d1, d2) => {
                if old_dir == d1 {
                    d2
                } else if old_dir == d2 {
                    d1
                } else {
                    panic!(
                        "got into pipe pos: {pos:?}, with old_dir; {old_dir:?}, which is not valid"
                    )
                }
            }
            _ => {
                unreachable!(
                    "got into non-pipe pos: {pos:?}, with old_dir; {old_dir:?}, which is not valid"
                )
            }
        }
    }

    fn is_connected(&self, start: (usize, usize), dir: Direction, dest: (usize, usize)) -> bool {
        use Tile::*;
        if start == dest {
            return false;
        }

        match (&self[start], dir, &self[dest]) {
            (_, dir, Pipe(exit1, exit2)) => *exit1 == !dir || *exit2 == !dir,
            (Pipe(exit1, exit2), dir, Start) => *exit1 == dir || *exit2 == dir,
            _ => false,
        }
    }
}

impl std::ops::Index<(usize, usize)> for Maze {
    type Output = Tile;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        let idx = row * self.cols + col;
        &self.tiles[idx]
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Tile {
    Ground,
    Start,
    Pipe(Direction, Direction),
}

impl TryFrom<char> for Tile {
    type Error = anyhow::Error;

    fn try_from(ch: char) -> std::prelude::v1::Result<Self, Self::Error> {
        use Direction::*;
        use Tile::*;
        Ok(match ch {
            '|' => Pipe(North, South),
            '-' => Pipe(East, West),
            'L' => Pipe(North, East),
            'J' => Pipe(North, West),
            '7' => Pipe(South, West),
            'F' => Pipe(South, East),
            '.' => Ground,
            'S' => Start,
            unknown => anyhow::bail!("unknown tile type: {unknown:?}"),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl std::ops::Not for Direction {
    type Output = Direction;

    fn not(self) -> Self::Output {
        use Direction::*;
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }
}

impl Direction {}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT1: &str = r#".....
.S-7.
.|.|.
.L-J.
....."#;

    const INPUT2: &str = r#"..F7.
.FJ|.
SJ.L7
|F--J
LJ..."#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT1)?, 4);
        assert_eq!(part_one(INPUT2)?, 8);
        Ok(())
    }
}
