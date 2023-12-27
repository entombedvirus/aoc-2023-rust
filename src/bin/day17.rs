use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};

use anyhow::{Context, Result};
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let p: Puzzle = input.parse()?;
    p.a_star((0, 0), (p.num_rows - 1, p.num_cols - 1), 1, 3)
        .context("path to goal not found")
}

fn part_two(input: &str) -> Result<u32> {
    let p: Puzzle = input.parse()?;
    p.a_star((0, 0), (p.num_rows - 1, p.num_cols - 1), 4, 10)
        .context("path to goal not found")
}

#[derive(Debug)]
struct Puzzle {
    num_cols: usize,
    num_rows: usize,
    nodes: Vec<u8>,
}

impl Puzzle {
    fn a_star(
        &self,
        start: (Row, Col),
        goal: (Row, Col),
        min_step: u8,
        max_step: u8,
    ) -> Option<u32> {
        let mut queue = BinaryHeap::new();
        let mut seen = BTreeSet::new();

        queue.push(Reverse(Node {
            cost: 0,
            pos: start,
            dir: Direction::Right,
            num_steps: 0,
        }));
        queue.push(Reverse(Node {
            cost: 0,
            pos: start,
            dir: Direction::Down,
            num_steps: 0,
        }));

        while let Some(Reverse(current)) = queue.pop() {
            if current.pos == goal {
                return Some(current.cost);
            }

            for (ndir, nr, nc) in self.neighbor(current.pos) {
                if ndir == !current.dir {
                    // can't go back the way we came
                    continue;
                }
                let num_steps = if ndir == current.dir {
                    current.num_steps + 1
                } else if current.num_steps >= min_step {
                    1
                } else {
                    continue;
                };
                if num_steps > max_step {
                    continue;
                }
                let neighbor_node = Node {
                    cost: current.cost + (self[(nr, nc)] - b'0') as u32,
                    pos: (nr, nc),
                    dir: ndir,
                    num_steps,
                };
                let key = neighbor_node.key();
                if !seen.contains(&key) {
                    queue.push(Reverse(neighbor_node));
                    seen.insert(key);
                }
            }
        }
        None
    }

    fn neighbor(&self, (row, col): (usize, usize)) -> Vec<(Direction, usize, usize)> {
        use Direction::*;
        let mut ret = Vec::new();
        if row > 0 {
            ret.push((Up, row - 1, col));
        }
        if row + 1 < self.num_rows {
            ret.push((Down, row + 1, col));
        }
        if col > 0 {
            ret.push((Left, row, col - 1));
        }
        if col + 1 < self.num_cols {
            ret.push((Right, row, col + 1));
        }
        ret
    }
}

impl std::str::FromStr for Puzzle {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let num_cols = s.lines().next().map(|l| l.len()).unwrap_or(0);
        let num_rows = s.lines().count();
        let nodes = s
            .as_bytes()
            .iter()
            .copied()
            .filter(|ch| *ch != b'\n')
            .collect();
        Ok(Self {
            num_cols,
            num_rows,
            nodes,
        })
    }
}

impl std::ops::Index<(usize, usize)> for Puzzle {
    type Output = u8;
    fn index(&self, (r, c): (usize, usize)) -> &Self::Output {
        let idx = r * self.num_cols + c;
        &self.nodes[idx]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Puzzle {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut Self::Output {
        let idx = r * self.num_cols + c;
        &mut self.nodes[idx]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Node {
    cost: u32,
    pos: (Row, Col),
    dir: Direction,
    num_steps: u8,
}

impl Node {
    fn key(&self) -> ((Row, Col), Direction, u8) {
        (self.pos, self.dir, self.num_steps)
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl std::ops::Not for Direction {
    type Output = Self;
    fn not(self) -> Self::Output {
        use Direction::*;
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Direction::*;
        match self {
            Up => write!(f, "↑"),
            Down => write!(f, "↓"),
            Left => write!(f, "←"),
            Right => write!(f, "→"),
        }
    }
}

type Row = usize;
type Col = usize;

#[derive(Debug)]
struct AstarState<'a>(
    &'a Puzzle,
    &'a BinaryHeap<Reverse<Node>>,
    &'a BTreeSet<((Row, Col), Direction, u8)>,
);

impl<'a> std::fmt::Display for AstarState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(puzzle, queue, seen) = self;
        let queue_nodes: BTreeMap<(Row, Col), Vec<&Node>> =
            queue
                .iter()
                .fold(BTreeMap::new(), |mut acc, Reverse(node)| {
                    acc.entry(node.pos)
                        .and_modify(|nodes| nodes.push(node))
                        .or_insert(vec![node]);
                    acc
                });
        for r in 0..puzzle.num_rows {
            for c in 0..puzzle.num_cols {
                if let Some(nodes) = queue_nodes.get(&(r, c)) {
                    write!(f, "[")?;
                    for n in nodes {
                        let key = n.key();
                        write!(f, "<{:<2},{},{:1}> ", n.cost, n.dir, n.num_steps)?;
                        if seen.contains(&key) {
                            write!(f, "*")?;
                        }
                    }
                    write!(f, "]")?;
                } else {
                    write!(f, "{:8} ", "x")?;
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

    const INPUT: &str = r#"2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 102);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 94);
        Ok(())
    }

    #[test]
    fn test_binary_heap() {
        let mut h = BinaryHeap::new();
        h.push(1);
        h.push(2);
        h.push(1);
        assert_eq!(h.len(), 3);
    }
}
// 2>>34^>>>1323 04 05       23 25 28 29
// 32v>>>35v5623    06 11 15 20       32
// 32552456v>>54                      37 41 43
// 3446585845v52                            47
// 4546657867v>6                            52 55
// 14385987984v4                               60
// 44578769877v6                               66
// 36378779796v>                               71 74
// 465496798688v                                  81
// 456467998645v                                  84
// 12246868655<v                                  93 87
// 25465488877v5                                  96
// 43226746555v>                                  99 102
