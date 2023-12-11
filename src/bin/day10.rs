#![feature(iter_map_windows)]

use std::fmt::Formatter;

use anyhow::{Context, Result};
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let maze = Maze::parse(input)?;
    Ok(maze.main_loop().count() / 2)
}

fn part_two(input: &str) -> Result<i32> {
    let maze = Maze::parse(input)?;

    let main_loop_coords: Vec<(usize, usize)> = maze.main_loop().collect();
    let num_boundary_points = main_loop_coords.len() as i32;
    let area = compute_area(&main_loop_coords);

    // See: https://en.wikipedia.org/wiki/Pick%27s_theorem
    let num_interior_points = area - (num_boundary_points / 2) + 1;
    Ok(num_interior_points)
}

// See: https://en.wikipedia.org/wiki/Shoelace_formula
fn compute_area(coords: &[(usize, usize)]) -> i32 {
    let first = coords
        .first()
        .expect("can't compute area without coordinates");
    let iter = coords
        .iter()
        .chain(std::iter::once(first))
        .map(|(row, col)| (*col as i32, *row as i32));
    iter.map_windows(|&[(x1, y1), (x2, y2)]| (x1 * y2) - (x2 * y1))
        .sum::<i32>()
        .abs()
        / 2
}

#[derive(Debug)]
struct Maze<T = Tile> {
    rows: usize,
    cols: usize,
    tiles: Vec<T>,
}

impl<T> std::fmt::Display for Maze<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.tiles.chunks_exact(self.cols) {
            for col in row {
                col.fmt(f)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Maze {
    fn parse(input: &str) -> Result<Self> {
        let cols = input
            .lines()
            .next()
            .map(|r| r.len())
            .context("input is empty")?;

        let mut rows = 0;
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

    fn valid_directions(&self, start: (usize, usize)) -> Vec<Direction> {
        Direction::ALL
            .iter()
            .copied()
            .filter(|d| {
                self.new_pos(start, *d)
                    .map(|new_pos| self.is_connected(start, *d, new_pos))
                    .unwrap_or(false)
            })
            .collect()
    }

    fn main_loop<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + 'a {
        let start = self.starting_pos();
        let starting_direction = self
            .valid_directions(start)
            .first()
            .copied()
            .expect("cannot move from starting position");
        self.navigate(start, starting_direction)
    }

    fn starting_pos(&self) -> (usize, usize) {
        let idx = self
            .tiles
            .iter()
            .position(|t| t == &Tile::Start)
            .expect("start position not found in maze");
        self.index_to_pos(idx)
    }
}

impl<T> Maze<T>
where
    T: PartialEq<T>,
{
    fn index_to_pos(&self, idx: usize) -> (usize, usize) {
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
}

impl<T> std::ops::Index<(usize, usize)> for Maze<T> {
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        let idx = row * self.cols + col;
        &self.tiles[idx]
    }
}

impl<T> std::ops::IndexMut<(usize, usize)> for Maze<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        let idx = row * self.cols + col;
        &mut self.tiles[idx]
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Tile {
    Ground,
    Start,
    Pipe(Direction, Direction),
}

impl Tile {
    fn as_char(&self) -> char {
        use Direction::*;
        use Tile::*;
        match self {
            Pipe(North, South) | Pipe(South, North) => '|',
            Pipe(East, West) | Pipe(West, East) => '-',
            Pipe(North, East) | Pipe(East, North) => 'L',
            Pipe(North, West) | Pipe(West, North) => 'J',
            Pipe(South, West) | Pipe(West, South) => '7',
            Pipe(South, East) | Pipe(East, South) => 'F',
            Ground => '.',
            Start => 'S',
            unknown => panic!("unsupported Tile config: {unknown:?}"),
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
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

impl Direction {
    const ALL: [Direction; 4] = [Self::North, Self::East, Self::South, Self::West];
}

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

    const INPUT3: &str = r#"...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
..........."#;

    const INPUT4: &str = r#"..........
.S------7.
.|F----7|.
.||....||.
.||....||.
.|L-7F-J|.
.|..||..|.
.L--JL--J.
.........."#;

    const INPUT5: &str = r#".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ..."#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT1)?, 4);
        assert_eq!(part_one(INPUT2)?, 8);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT3)?, 4);
        assert_eq!(part_two(INPUT4)?, 4);
        assert_eq!(part_two(INPUT5)?, 8);
        Ok(())
    }
}
