#![feature(iter_map_windows)]

use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{
    bytes::complete::{is_a, tag},
    character::complete::{self, newline, one_of, space1},
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, tuple},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u64> {
    let p = Puzzle::parse(input)?;
    Ok(compute_lava_cubes(&p.as_coords()))
}

fn part_two(input: &str) -> Result<u64> {
    let p = Puzzle::parse(input)?;
    Ok(compute_lava_cubes(&p.as_corrected().as_coords()))
}

fn compute_lava_cubes(coords: &[(i64, i64)]) -> u64 {
    let area = compute_area_via_shoelace(coords);
    let b = num_boundary_points(coords);
    // See: https://en.wikipedia.org/wiki/Pick's_theorem
    let i = (area + 1) - b / 2;
    i + b
}

#[derive(Debug)]
struct Puzzle<'i> {
    ins: Vec<Instruction<'i>>,
}
impl<'i> Puzzle<'i> {
    fn parse(input: &'i str) -> Result<Self> {
        let parse_direction = map(one_of("UDRL"), |ch: char| match ch {
            'U' => Direction::Up,
            'D' => Direction::Down,
            'L' => Direction::Left,
            'R' => Direction::Right,
            other => unreachable!("unexpected direction: {}", other),
        });
        let parse_color = delimited(tag("(#"), is_a("0123456789abcdef"), complete::char(')'));
        let parse_ins = map(
            tuple((parse_direction, space1, complete::u64, space1, parse_color)),
            |(dir, _, count, _, color)| Instruction { dir, count, color },
        );
        let parse_instructions = separated_list1(newline, parse_ins);
        let parser = map(parse_instructions, |ins: Vec<Instruction>| Self { ins });
        must_parse(parser, input)
    }

    fn as_coords(&self) -> Vec<(i64, i64)> {
        let iter = self.ins.iter().scan((0, 0), |(x, y), ins| {
            let (nx, ny) = match ins.dir {
                Direction::Up => (*x, *y + ins.count as i64),
                Direction::Down => (*x, *y - ins.count as i64),
                Direction::Left => (*x - ins.count as i64, *y),
                Direction::Right => (*x + ins.count as i64, *y),
            };
            *x = nx;
            *y = ny;
            Some((nx, ny))
        });
        let mut ret = vec![(0, 0)];
        ret.extend(iter);
        ret
    }

    fn as_corrected(&self) -> Self {
        let ins = self
            .ins
            .iter()
            .map(|ins| {
                let count = u64::from_str_radix(&ins.color[..5], 16).expect("dist parsing failed");
                let dir = match ins.color.as_bytes()[5] {
                    b'0' => Direction::Right,
                    b'1' => Direction::Down,
                    b'2' => Direction::Left,
                    b'3' => Direction::Up,
                    other => unreachable!("unexpected direction: {}", other),
                };
                Instruction {
                    dir,
                    count,
                    color: ins.color,
                }
            })
            .collect();
        Self { ins }
    }
}

#[derive(Debug)]
struct Instruction<'i> {
    dir: Direction,
    count: u64,
    color: &'i str,
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// See: https://en.wikipedia.org/wiki/Shoelace_formula
fn compute_area_via_shoelace(coords: &[(i64, i64)]) -> u64 {
    assert!(coords.first() == coords.last());
    coords
        .iter()
        .map_windows(|&[(x1, y1), (x2, y2)]| (x1 * y2) - (x2 * y1))
        .sum::<i64>()
        .abs() as u64
        / 2
}

fn num_boundary_points(points: &[(i64, i64)]) -> u64 {
    points
        .iter()
        .map_windows(|&[(x1, y1), (x2, y2)]| x1.abs_diff(*x2) + y1.abs_diff(*y2))
        .sum::<u64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 62);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 952408144115);
        Ok(())
    }

    #[test]
    fn test_coords() -> Result<()> {
        let p = Puzzle::parse(INPUT)?;
        let _coords = p.as_coords();
        Ok(())
    }
}
