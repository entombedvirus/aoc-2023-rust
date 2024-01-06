#![allow(unused, dead_code)]

use std::{collections::btree_map::Range, fmt::Formatter};

use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{
    character::complete::{self, newline},
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let mut p = Puzzle::parse(input)?;
    eprintln!("before fall:\n{p}");
    p.fall();
    eprintln!("after fall:\n{p}");
    todo!()
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Puzzle {
    bricks: Vec<Brick>,
}

impl Puzzle {
    fn parse(input: &str) -> Result<Self> {
        let parse_vec3 = || {
            map(
                tuple((
                    complete::i32,
                    complete::char(','),
                    complete::i32,
                    complete::char(','),
                    complete::i32,
                )),
                |(x, _, y, _, z)| Vec3 { x, y, z },
            )
        };
        let parse_brick = map(
            separated_pair(parse_vec3(), complete::char('~'), parse_vec3()),
            |(p1, p2)| Brick { p1, p2 },
        );
        let parser = map(separated_list1(newline, parse_brick), |bricks| Self {
            bricks,
        });
        must_parse(parser, input)
    }

    fn fall(&mut self) {
        let mut bricks = &mut self.bricks;
        if bricks.is_empty() {
            return;
        }
        bricks.sort_unstable_by_key(|b| b.z_min());
        let mut i = 0;
        while i < bricks.len() {
            let intersecting_brick = bricks.get(0..i).and_then(|lower_bricks| {
                lower_bricks
                    .iter()
                    .rev()
                    .find(|b| b.intersects_xy(&bricks[i]))
            });
            bricks[i].move_down_to_z(match intersecting_brick {
                None => 1, // brick can go all the way to the ground
                Some(intersecting_brick) => intersecting_brick.z_max() + 1,
            });
            i += 1;
        }
    }
}

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for b in self.bricks.iter().rev() {
            writeln!(
                f,
                "Brick({} -> {}, {} -> {}, {} -> {})",
                b.p1.x, b.p2.x, b.p1.y, b.p2.y, b.p1.z, b.p2.z
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Brick {
    p1: Vec3,
    p2: Vec3,
}
impl Brick {
    fn z_min(&self) -> i32 {
        std::cmp::min(self.p1.z, self.p2.z)
    }

    fn z_max(&self) -> i32 {
        std::cmp::max(self.p1.z, self.p2.z)
    }

    fn move_down_to_z(&self, z: i32) -> Brick {
        let diff = self.z_min().saturating_sub(z);
        let mut t = self.clone();
        t.p1.z -= diff;
        t.p2.z -= diff;
        t
    }

    fn intersects_xy(&self, other: &Brick) -> bool {
        let x = self.x_range();
        let y = self.y_range();
        (x.contains(&other.p1.x) || x.contains(&other.p2.x))
            && (y.contains(&other.p1.y) || y.contains(&other.p2.y))
    }

    fn y_range(&self) -> std::ops::RangeInclusive<i32> {
        if self.p1.y < self.p2.y {
            self.p1.y..=self.p2.y
        } else {
            self.p2.y..=self.p1.y
        }
    }

    fn x_range(&self) -> std::ops::RangeInclusive<i32> {
        if self.p1.x < self.p2.x {
            self.p1.x..=self.p2.x
        } else {
            self.p2.x..=self.p1.x
        }
    }
}

#[derive(Debug, Clone)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 5);
        Ok(())
    }

    #[test]
    fn test_intersects_xy() {
        let b1 = Brick {
            p1: Vec3 { x: 0, y: 0, z: 0 },
            p2: Vec3 { x: 2, y: 0, z: 0 },
        };
        let b2 = Brick {
            p1: Vec3 { x: 2, y: 0, z: 10 },
            p2: Vec3 { x: 6, y: 0, z: 10 },
        };
        assert!(b1.intersects_xy(&b2));
    }
}
