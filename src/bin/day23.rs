#![allow(unused, dead_code)]
#![feature(extract_if)]
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::Result;
use aoc::{must_parse, runner, wait};
use nom::{character::complete::newline, multi::separated_list1};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let p = Puzzle::parse(input)?;
    let start = p.start_pos().ok_or(anyhow::format_err!("no start pos"))?;
    let finish = p.finish_pos().ok_or(anyhow::format_err!("no finish pos"))?;

    p.longest_path(start, finish)
        .ok_or(anyhow::format_err!("path to finish not found"))
}

fn part_two(input: &str) -> Result<u32> {
    let mut p = Puzzle::parse(input)?;
    // treat all slides as normal tiles
    for tile in &mut p.tiles {
        match *tile {
            b'>' | b'<' | b'^' | b'v' => *tile = b'.',
            _ => (),
        }
    }

    let start = p.start_pos().ok_or(anyhow::format_err!("no start pos"))?;
    let finish = p.finish_pos().ok_or(anyhow::format_err!("no finish pos"))?;
    let graph = p.as_graph(start);
    eprintln!("{graph}");

    let mut longest_path = None;
    let mut q = VecDeque::new();
    q.push_back((start, BTreeSet::new(), String::new(), 0u32));

    while let Some((cur_node, mut seen, mut path, cost)) = q.pop_back() {
        seen.insert(cur_node);
        if !path.is_empty() {
            path.push_str(format!(" -{cost}-> ").as_str())
        }
        path.push_str(format!("{cur_node:?}").as_str());
        if cur_node == finish {
            longest_path = std::cmp::max(longest_path, Some(cost));
            continue;
        }
        for (neighbor, ncost) in graph.edges_from(cur_node) {
            if !seen.contains(&neighbor) {
                q.push_back((neighbor, seen.clone(), path.clone(), cost + ncost));
            }
        }
    }
    longest_path.ok_or(anyhow::format_err!("path to finish not found"))
}

type Pos = (isize, isize);

#[derive(Debug, Default)]
struct Graph {
    neighbors: BTreeMap<Pos, BTreeSet<Pos>>,
    edges: BTreeMap<(Pos, Pos), u32>,
}

impl Graph {
    fn add_edge(&mut self, n1: Pos, n2: Pos, cost: u32) {
        self.edges.entry((n1, n2)).or_insert(cost);
        self.edges.entry((n2, n1)).or_insert(cost);
        self.neighbors.entry(n1).or_default().insert(n2);
        self.neighbors.entry(n2).or_default().insert(n1);
    }

    fn edges_from<'a>(&'a self, cur_node: (isize, isize)) -> impl Iterator<Item = (Pos, u32)> + 'a {
        self.neighbors[&cur_node]
            .iter()
            .map(move |neighbor| (*neighbor, self.edges[&(cur_node, *neighbor)]))
        // self.edges
        //     .iter()
        //     .skip_while(move |((from, _), _)| *from != cur_node)
        //     .take_while(move |((from, _), _)| *from == cur_node)
        //     .map(|((_from, to), cost)| (*to, *cost))
    }
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Graph:")?;
        for ((from, to), cost) in &self.edges {
            writeln!(
                f,
                "\t{from:?} --{cost}--> {to:?}, neighbors: {:?}",
                self.neighbors.get(from),
            )?;
        }
        Ok(())
    }
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

    fn move_to(&mut self, pos: &(isize, isize), cost_delta: u32) {
        let cost = self.seen[&self.cur_pos] + cost_delta;
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

    fn get_neighbor_pos(&self, cur_pos: (isize, isize)) -> Vec<(isize, isize)> {
        let mut npos = Vec::new();
        let mut push_if_valid = |pos| {
            match self.get_tile(pos) {
                Some(b'.' | b'<' | b'>' | b'^' | b'v') => npos.push(pos),
                _ => (),
            };
        };
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

    fn longest_path(&self, start: (isize, isize), finish: (isize, isize)) -> Option<u32> {
        let mut longest_hike = None;
        let mut q = VecDeque::new();
        q.push_back(Head::new(start, 0));

        while let Some(mut head) = q.pop_front() {
            if head.cur_pos == finish {
                longest_hike = std::cmp::max(longest_hike, head.cost());
                continue;
            }
            let mut neighbors = self.get_neighbor_pos(head.cur_pos);
            while let Some(npos) = neighbors.pop() {
                if head.seen.contains_key(&npos) {
                    continue;
                }
                let mut head = if neighbors.is_empty() {
                    std::mem::take(&mut head)
                } else {
                    head.clone()
                };
                head.move_to(&npos, 1);
                q.push_back(head);
            }
        }

        longest_hike
    }

    fn as_graph(&self, start: (isize, isize)) -> Graph {
        let mut graph = Graph::default();
        let mut q = Vec::new();
        q.push((start, (start.0 + 1, start.1)));

        let mut seen = BTreeSet::new();
        seen.insert(start);

        while let Some((segment_start_pos, mut cur_pos)) = q.pop() {
            let mut cost = 0u32;
            loop {
                seen.insert(cur_pos);
                cost += 1;
                let mut neighbor_positions = self.get_neighbor_pos(cur_pos);
                neighbor_positions.retain(|pos| !seen.contains(pos));
                match &neighbor_positions[..] {
                    [] => {
                        if cost > 1 {
                            graph.add_edge(segment_start_pos, cur_pos, cost);
                        }
                        break;
                    }
                    [npos] => {
                        if let Some((pending_start_pos, _)) = q.iter().find(|(_, ps)| ps == npos) {
                            graph.add_edge(segment_start_pos, *pending_start_pos, cost + 2);
                            break;
                        }
                        cur_pos = *npos;
                        continue;
                    }
                    many_npos => {
                        // cost += 1;
                        graph.add_edge(segment_start_pos, cur_pos, cost);
                        q.extend(many_npos.iter().copied().map(|npos| (cur_pos, npos)));
                        break;
                    }
                }
            }
        }

        graph
    }

    // fn debug_print(
    //     &self,
    //     q: &[(Node, (isize, isize))],
    //     node: &Node,
    //     cur_pos: (isize, isize),
    //     seen: &BTreeSet<(isize, isize)>,
    // ) {
    //     for r in 0..self.num_rows {
    //         for c in 0..self.num_cols {
    //             let pos = (r as isize, c as isize);
    //             let ch = if q.iter().find(|(n, _)| n.pos == pos).is_some() {
    //                 'P'
    //             } else if seen.contains(&pos) {
    //                 'x'
    //             } else {
    //                 char::from(self.tiles[r * self.num_cols + c])
    //             };
    //             eprint!("{ch}");
    //         }
    //         eprintln!();
    //     }
    // }
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

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 154);
        Ok(())
    }
}
