use std::{
    collections::BTreeSet,
    fmt::{Formatter, Write},
};

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

fn part_one(input: &str) -> Result<usize> {
    let mut p = Puzzle::parse(input)?;
    p.fall();
    Ok(p.disintegratable_bricks().count())
}

fn part_two(input: &str) -> Result<u32> {
    let mut p = Puzzle::parse(input)?;
    p.fall();
    Ok(p.chain_fall())
}

#[derive(Debug, Clone)]
struct Puzzle {
    bricks: BTreeSet<Brick>,
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
            |(p1, p2)| Brick { start: p1, end: p2 },
        );
        let parser = map(separated_list1(newline, parse_brick), Self::new);
        must_parse(parser, input)
    }

    fn new(bricks: Vec<Brick>) -> Self {
        Self {
            bricks: BTreeSet::from_iter(bricks),
        }
    }

    fn fall(&mut self) -> u32 {
        // sorted because BTreeSet::into_iter is sorted
        let mut sorted_bricks: Vec<_> = std::mem::take(&mut self.bricks).into_iter().collect();
        let mut fallen_bricks = 0;
        for i in 0..sorted_bricks.len() {
            // find the highest z value for from the list of already fallen
            // bricks that intersects with current brick
            let highest_z = sorted_bricks[0..i]
                .iter()
                .filter_map(|b| b.intersects_xy(&sorted_bricks[i]).then_some(b.z_max()))
                .max()
                .unwrap_or(0);
            if sorted_bricks[i].move_down_to_z(highest_z + 1) {
                fallen_bricks += 1;
            }
        }
        self.bricks.extend(sorted_bricks);
        fallen_bricks
    }

    // returns the number of bricks that will fall if each brick is removed
    fn chain_fall(&self) -> u32 {
        let mut count = 0;
        for cur_brick in &self.bricks {
            let mut p = self.clone();
            p.bricks.remove(cur_brick);
            count += p.fall();
        }
        count
    }

    fn disintegratable_bricks<'a>(&'a self) -> impl Iterator<Item = &'a Brick> + 'a {
        self.bricks.iter().filter(|b| self.can_remove_brick(b))
    }

    fn can_remove_brick(&self, brick: &Brick) -> bool {
        self.just_above(brick)
            .all(|b| self.just_below(b).count() > 1)
    }

    fn just_above<'a>(&'a self, brick: &'a Brick) -> impl Iterator<Item = &'a Brick> + 'a {
        let just_above = brick.z_max().saturating_add(1);
        self.bricks
            .iter()
            .filter(move |a| a.z_min() == just_above && a.intersects_xy(brick))
    }

    fn just_below<'a>(&'a self, brick: &'a Brick) -> impl Iterator<Item = &'a Brick> + 'a {
        let just_below = brick.z_min().saturating_sub(1);
        self.bricks
            .iter()
            .filter(move |a| a.z_max() == just_below && a.intersects_xy(brick))
    }

    #[allow(unused)]
    fn as_c_array(&self) -> String {
        let mut buf = String::new();
        writeln!(&mut buf, "Brick bricks[] = {{");
        for b in &self.bricks {
            writeln!(&mut buf, "(Brick){{.start = (Vector3){{.x = {}, .y = {}, .z = {} }}, .end =(Vector3){{.x = {}, .y = {}, .z = {} }} }},", b.start.x, b.start.y, b.start.z, b.end.x, b.end.y, b.end.z);
        }
        writeln!(&mut buf, "}};");
        buf
    }
}

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for b in self.bricks.iter().rev() {
            writeln!(f, "{b}",)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Brick {
    start: Vec3,
    end: Vec3,
}

impl std::cmp::PartialOrd for Brick {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Brick {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_z = self.z_min();
        let other_z = other.z_min();
        self_z.cmp(&other_z).then_with(|| {
            self.start
                .cmp(&other.start)
                .then_with(|| self.end.cmp(&other.end))
        })
    }
}

impl std::fmt::Display for Brick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Brick(")?;
        let mut write_range = |start: i32, end: i32, comma: bool| {
            if comma {
                write!(f, ", ")?;
            }
            if start == end {
                write!(f, "{start}")
            } else {
                write!(f, "{start} -> {end}")
            }
        };
        write_range(self.start.x, self.end.x, false)?;
        write_range(self.start.y, self.end.y, true)?;
        write_range(self.start.z, self.end.z, true)?;
        write!(f, ")")
    }
}

impl Brick {
    fn z_min(&self) -> i32 {
        std::cmp::min(self.start.z, self.end.z)
    }

    fn z_max(&self) -> i32 {
        std::cmp::max(self.start.z, self.end.z)
    }

    fn move_down_to_z(&mut self, z: i32) -> bool {
        let diff = self.z_min().saturating_sub(z);
        self.start.z -= diff;
        self.end.z -= diff;
        diff > 0
    }

    fn intersects_xy(&self, other: &Brick) -> bool {
        let x_intersects = self.end.x >= other.start.x && other.end.x >= self.start.x;
        let y_intersects = self.end.y >= other.start.y && other.end.y >= self.start.y;
        x_intersects && y_intersects
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 7);
        Ok(())
    }

    #[test]
    fn test_intersects_xy() {
        let b1 = Brick {
            start: Vec3 { x: 0, y: 0, z: 0 },
            end: Vec3 { x: 2, y: 0, z: 0 },
        };
        let b2 = Brick {
            start: Vec3 { x: 2, y: 0, z: 10 },
            end: Vec3 { x: 6, y: 0, z: 10 },
        };
        assert!(b1.intersects_xy(&b2));
    }
}
