#![allow(unused, dead_code)]

use std::collections::{BTreeMap, VecDeque};

use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{character::complete::newline, multi::separated_list1};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let p = Puzzle::parse(input)?;
    let start = p.start_pos().ok_or(anyhow::format_err!("no start pos"))?;
    let finish = p.finish_pos().ok_or(anyhow::format_err!("no finish pos"))?;

    let mut longest_hike = None;
    let mut q = VecDeque::new();
    q.push_back(Head::new(start, 0));
    while let Some(mut head) = q.pop_front() {
        if head.cur_pos == finish {
            longest_hike = std::cmp::max(longest_hike, head.cost());
        }
        let mut neighbors = p.get_neighbor_pos(&head);
        while let Some(npos) = neighbors.pop() {
            let mut head = if neighbors.is_empty() {
                std::mem::take(&mut head)
            } else {
                head.clone()
            };
            head.move_to(&npos);
            q.push_back(head);
        }
    }

    longest_hike.ok_or(anyhow::format_err!("path to finish not found"))
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug, Clone, Default)]
struct Head {
    seen: BTreeMap<(isize, isize), u32>,
    cur_pos: (isize, isize),
}

impl Head {
    fn new(cur_pos: (isize, isize), cost: u32) -> Self {
        let mut seen = BTreeMap::default();
        seen.insert(cur_pos, cost);
        Self { seen, cur_pos }
    }

    fn move_to(&mut self, pos: &(isize, isize)) {
        let cost = self.seen[&self.cur_pos] + 1;
        self.seen.entry(*pos).or_insert(cost);
        self.cur_pos = *pos;
    }

    fn cost(&self) -> Option<u32> {
        self.seen.get(&self.cur_pos).copied()
    }
}

#[derive(Debug)]
struct Puzzle {
    num_rows: usize,
    num_cols: usize,
    tiles: Vec<u8>,
}

impl Puzzle {
    fn parse(input: &str) -> Result<Self> {
        let mut num_rows = 0;
        let mut lines = input
            .lines()
            .map(|l| {
                num_rows += 1;
                l
            })
            .peekable();
        let num_cols = lines
            .peek()
            .ok_or(anyhow::format_err!("empty input"))?
            .len();

        let mut tiles = Vec::new();
        for line in lines {
            tiles.extend_from_slice(line.as_bytes());
        }

        Ok(Self {
            num_rows,
            num_cols,
            tiles,
        })
    }

    fn start_pos(&self) -> Option<(isize, isize)> {
        self.tiles.get(0..self.num_cols).and_then(|first_line| {
            first_line
                .iter()
                .position(|&ch| ch == b'.')
                .map(|idx| (0, idx as isize))
        })
    }

    fn finish_pos(&self) -> Option<(isize, isize)> {
        self.tiles
            .get(self.num_rows * self.num_cols - self.num_cols..)
            .and_then(|last_line| {
                last_line
                    .iter()
                    .position(|&ch| ch == b'.')
                    .map(|idx| ((self.num_rows - 1) as isize, idx as isize))
            })
    }

    fn get_tile(&self, (row, col): (isize, isize)) -> Option<u8> {
        if (0..self.num_rows).contains(&(row as usize))
            && (0..self.num_cols).contains(&(col as usize))
        {
            let idx = row as usize * self.num_cols + col as usize;
            self.tiles.get(idx).copied()
        } else {
            None
        }
    }

    fn get_neighbor_pos(&self, head: &Head) -> Vec<(isize, isize)> {
        let mut npos = Vec::new();
        let mut push_if_valid = |pos| {
            match self.get_tile(pos) {
                Some(b'.' | b'<' | b'>' | b'^' | b'v') if !head.seen.contains_key(&pos) => {
                    npos.push(pos)
                }
                _ => (),
            };
        };
        let cur_pos = head.cur_pos;
        match self.get_tile(cur_pos) {
            Some(b'>') => push_if_valid((cur_pos.0, cur_pos.1 + 1)),
            Some(b'<') => push_if_valid((cur_pos.0, cur_pos.1 - 1)),
            Some(b'^') => push_if_valid((cur_pos.0 - 1, cur_pos.1)),
            Some(b'v') => push_if_valid((cur_pos.0 + 1, cur_pos.1)),
            Some(b'.') => [(0, 1), (0, -1), (-1, 0), (1, 0)]
                .into_iter()
                .for_each(|(dr, dc)| push_if_valid((cur_pos.0 + dr, cur_pos.1 + dc))),

            Some(b'#') => (),
            unknown => panic!("unknown tile {:?}", unknown),
        };
        npos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 94);
        Ok(())
    }
}
