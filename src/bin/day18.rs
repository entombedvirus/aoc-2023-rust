#![feature(iter_map_windows)]
#![allow(unused, dead_code)]

use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{
    bytes::complete::{is_a, tag},
    character::complete::{self, hex_digit1, newline, one_of, space1},
    combinator::{map, map_res},
    multi::separated_list1,
    number::complete::hex_u32,
    sequence::{delimited, preceded, tuple},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let p = Puzzle::parse(input)?;
    let coords = p.as_coords();
    let area = compute_area_via_shoelace(coords.as_slice());
    let b = num_boundary_points(coords.as_slice());
    // See: https://en.wikipedia.org/wiki/Pick's_theorem
    let i = (area + 1) - b / 2;
    Ok(i + b)
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Puzzle {
    ins: Vec<Instruction>,
}
impl Puzzle {
    fn parse(input: &str) -> Result<Self> {
        let parse_direction = map(one_of("UDRL"), |ch: char| match ch {
            'U' => Direction::Up,
            'D' => Direction::Down,
            'L' => Direction::Left,
            'R' => Direction::Right,
            other => unreachable!("unexpected direction: {}", other),
        });
        let parse_hex = map_res(is_a("0123456789abcdef"), |as_str: &str| {
            u32::from_str_radix(as_str, 16)
        });
        let parse_color = delimited(tag("(#"), parse_hex, complete::char(')'));
        let parse_ins = map(
            tuple((parse_direction, space1, complete::u8, space1, parse_color)),
            |(dir, _, count, _, color)| Instruction { dir, count, color },
        );
        let parse_instructions = separated_list1(newline, parse_ins);
        let mut parser = map(parse_instructions, |ins: Vec<Instruction>| Self { ins });
        must_parse(parser, input)
    }

    fn as_coords(&self) -> Vec<(i32, i32)> {
        let iter = self.ins.iter().scan((0, 0), |(x, y), ins| {
            let (nx, ny) = match ins.dir {
                Direction::Up => (*x, *y + ins.count as i32),
                Direction::Down => (*x, *y - ins.count as i32),
                Direction::Left => (*x - ins.count as i32, *y),
                Direction::Right => (*x + ins.count as i32, *y),
            };
            *x = nx;
            *y = ny;
            Some((nx, ny))
        });
        let mut ret = vec![(0, 0)];
        ret.extend(iter);
        ret
    }
}

#[derive(Debug)]
struct Instruction {
    dir: Direction,
    count: u8,
    color: u32,
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// See: https://en.wikipedia.org/wiki/Shoelace_formula
fn compute_area_via_shoelace(coords: &[(i32, i32)]) -> u32 {
    assert!(coords.first() == coords.last());
    coords
        .iter()
        .map_windows(|&[(x1, y1), (x2, y2)]| (x1 * y2) - (x2 * y1))
        .sum::<i32>()
        .abs() as u32
        / 2
}

fn num_boundary_points(points: &[(i32, i32)]) -> u32 {
    points
        .iter()
        .map_windows(|&[(x1, y1), (x2, y2)]| x1.abs_diff(*x2) + y1.abs_diff(*y2))
        .sum::<u32>()
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
    fn test_coords() -> Result<()> {
        let p = Puzzle::parse(INPUT)?;
        let coords = p.as_coords();
        Ok(())
    }
}
