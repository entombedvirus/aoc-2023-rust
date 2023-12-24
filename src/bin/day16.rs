use anyhow::{Context, Result};
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let p: Puzzle = input.parse()?;
    let start = Head {
        pos: (0, 0),
        heading: Direction::Right,
    };
    let mask = p.energize(start);
    Ok(mask.num_energized())
}

fn part_two(input: &str) -> Result<usize> {
    let p: Puzzle = input.parse()?;
    p.start_positions()
        .map(|head| p.energize(head).num_energized())
        .max()
        .context("no start positions")
}

#[derive(Debug)]
struct Puzzle {
    num_cols: usize,
    num_rows: usize,
    tiles: Vec<u8>,
}

impl Puzzle {
    fn energize(&self, start: Head) -> TileMask<'_> {
        let mut mask = TileMask {
            puzzle: self,
            directions: vec![Vec::new(); self.tiles.len()],
        };

        let mut heads = vec![start];
        while !heads.is_empty() {
            heads.retain(|head| {
                let Some(valid_pos) = self.validate_index(head.pos.0, head.pos.1) else {
                    return false;
                };
                if mask[valid_pos].contains(&head.heading) {
                    return false;
                } else {
                    mask[valid_pos].push(head.heading);
                    return true;
                }
            });
            let new_heads = heads
                .iter_mut()
                .filter_map(|head| head.step(self))
                .collect::<Vec<_>>();
            heads.extend(new_heads);
        }
        mask
    }

    fn validate_index(&self, row: isize, col: isize) -> Option<(usize, usize)> {
        let row: usize = row.try_into().ok()?;
        let col: usize = col.try_into().ok()?;
        if row < self.num_rows && col < self.num_cols {
            Some((row, col))
        } else {
            None
        }
    }

    fn start_positions(&self) -> impl Iterator<Item = Head> {
        let num_cols = self.num_cols as isize;
        let num_rows = self.num_rows as isize;
        (0..num_cols)
            .map(|c| Head {
                pos: (0, c),
                heading: Direction::Down,
            })
            .chain((0..num_cols).map(move |c| Head {
                pos: (num_rows - 1, c),
                heading: Direction::Up,
            }))
            .chain((0..num_rows).map(move |r| Head {
                pos: (r, 0),
                heading: Direction::Right,
            }))
            .chain((0..num_rows).map(move |r| Head {
                pos: (r, num_cols - 1),
                heading: Direction::Left,
            }))
    }
}

impl std::str::FromStr for Puzzle {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let num_rows = input.lines().count();
        let num_cols = input.lines().next().map(|l| l.len()).unwrap_or(0);
        let tiles = input
            .as_bytes()
            .iter()
            .copied()
            .filter(|ch| *ch != b'\n')
            .collect();
        Ok(Self {
            num_rows,
            num_cols,
            tiles,
        })
    }
}

type Row = usize;
type Col = usize;
impl std::ops::Index<(Row, Col)> for Puzzle {
    type Output = u8;

    fn index(&self, (r, c): (Row, Col)) -> &Self::Output {
        let idx = r * self.num_cols + c;
        &self.tiles[idx]
    }
}

impl std::ops::IndexMut<(Row, Col)> for Puzzle {
    fn index_mut(&mut self, (r, c): (Row, Col)) -> &mut Self::Output {
        let idx = r * self.num_cols + c;
        &mut self.tiles[idx]
    }
}

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.num_rows {
            for c in 0..self.num_cols {
                write!(f, "{}", self[(r, c)] as char)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct TileMask<'p> {
    puzzle: &'p Puzzle,
    directions: Vec<Vec<Direction>>,
}
impl TileMask<'_> {
    fn num_energized(&self) -> usize {
        self.directions.iter().filter(|dirs| dirs.len() > 0).count()
    }
}

impl std::fmt::Display for TileMask<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.puzzle.num_rows {
            for c in 0..self.puzzle.num_cols {
                match self.puzzle[(r, c)] {
                    b'.' => {
                        let idx = r * self.puzzle.num_cols + c;
                        let dirs = &self.directions[idx];
                        match dirs.len().cmp(&1) {
                            std::cmp::Ordering::Less => write!(f, "."),
                            std::cmp::Ordering::Equal => write!(f, "{}", dirs[0]),
                            std::cmp::Ordering::Greater => write!(f, "{}", dirs.len()),
                        }?
                    }
                    tile => {
                        write!(f, "{}", tile as char)?;
                    }
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl std::ops::Index<(Row, Col)> for TileMask<'_> {
    type Output = Vec<Direction>;

    fn index(&self, (r, c): (Row, Col)) -> &Self::Output {
        let idx = r * self.puzzle.num_cols + c;
        &self.directions[idx]
    }
}

impl std::ops::IndexMut<(Row, Col)> for TileMask<'_> {
    fn index_mut(&mut self, (r, c): (Row, Col)) -> &mut Self::Output {
        let idx = r * self.puzzle.num_cols + c;
        &mut self.directions[idx]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
struct Head {
    pos: (isize, isize),
    heading: Direction,
}

impl Head {
    fn step(&mut self, puzzle: &Puzzle) -> Option<Self> {
        use Direction::*;
        let hrow = &mut self.pos.0;
        let hcol = &mut self.pos.1;
        let mut new_head = None;

        let Some(valid_pos) = puzzle.validate_index(*hrow, *hcol) else {
            return None;
        };
        self.heading = match (puzzle[valid_pos], self.heading) {
            (b'\\', Up) => Left,
            (b'\\', Down) => Right,
            (b'\\', Left) => Up,
            (b'\\', Right) => Down,
            (b'/', Up) => Right,
            (b'/', Down) => Left,
            (b'/', Left) => Down,
            (b'/', Right) => Up,
            (b'-', Up | Down) => {
                new_head = Some(Self {
                    pos: (*hrow, *hcol + 1),
                    heading: Right,
                });
                Left
            }
            (b'|', Left | Right) => {
                new_head = Some(Self {
                    pos: (*hrow + 1, *hcol),
                    heading: Down,
                });
                Up
            }
            _other => self.heading,
        };
        match self.heading {
            Up => *hrow -= 1,
            Down => *hrow += 1,
            Left => *hcol -= 1,
            Right => *hcol += 1,
        };
        new_head
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Direction::*;
        let ch = match self {
            Up => '↑',
            Down => '↓',
            Left => '←',
            Right => '→',
        };
        write!(f, "{}", ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|...."#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 46);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 51);
        Ok(())
    }
}
