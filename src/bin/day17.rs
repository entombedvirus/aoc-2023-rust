#![allow(unused, dead_code)]

use std::collections::{BTreeMap, BinaryHeap, VecDeque};

use anyhow::Result;
use aoc::{runner, wait};

// 3 blocks means one less direction
const MAX_SAME_DIRS: usize = 3;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let p: Puzzle = input.parse()?;
    Ok(p.a_star((0, 0), (p.num_rows - 1, p.num_cols - 1)))
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Puzzle {
    num_cols: usize,
    num_rows: usize,
    nodes: Vec<u8>,
}

impl Puzzle {
    fn a_star(&self, start: (usize, usize), goal: (usize, usize)) -> usize {
        // heuristic for estimating distance between passed in node and dest
        // here, I am using the Manhattan Distance between the two nodes
        let h = |(r, c): (usize, usize)| r.abs_diff(goal.0) + c.abs_diff(goal.1);

        let start_node = Node {
            pos: start,
            est_cost: h(start),
            last_dir: Direction::Right,
            moves_left: MAX_SAME_DIRS as u8,
        };
        // The set of discovered nodes that may need to be (re-)expanded.
        // Initially, only the start node is known.
        let mut open_set = BinaryHeap::new();
        open_set.push(start_node.clone());

        // For node n, costs[n] is the cost of the cheapest path from start to n currently known.
        let mut costs = BTreeMap::new();
        costs.insert(start_node.clone(), 0);

        // For node n, parent[n.pos] is the previous node in the optimal path from start to goal
        let mut parent = BTreeMap::new();

        while let Some(current) = open_set.pop() {
            for (dir, nr, nc) in self.neighbor(current.pos) {
                // can't go straight for too long
                let is_allowed = |src: &Node, new_dir: Direction, neighbor: (Row, Col)| {
                    if parent.get(src) == Some(&neighbor) {
                        false
                    } else {
                        !(src.last_dir == new_dir && src.moves_left == 0)
                        // let dirs = &prev_dirs[&src];
                        // dirs.len() < MAX_SAME_DIRS
                        //     || dirs
                        //         .range(dirs.len() - MAX_SAME_DIRS..)
                        //         .any(|d| d != &new_dir)
                    }
                };

                let n = (nr, nc);
                if !is_allowed(&current, dir, n) {
                    // can't consider this neighbor because we have already used up the at-most 3
                    // blocks rule slots
                    continue;
                }
                let tenatative = costs[&current] + (self[n] - b'0') as usize;
                let mut cost_updated = false;
                let neighbor_node = Node {
                    pos: n,
                    est_cost: tenatative + h(n),
                    last_dir: dir,
                    moves_left: if current.last_dir == dir {
                        current.moves_left - 1
                    } else {
                        3
                    },
                };
                costs
                    .entry(neighbor_node.clone())
                    .and_modify(|prev_cost| {
                        if tenatative < *prev_cost {
                            cost_updated = true;
                            *prev_cost = tenatative;
                        }
                    })
                    .or_insert_with(|| {
                        cost_updated = true;
                        tenatative
                    });
                if cost_updated {
                    if open_set
                        .iter()
                        .find(|n| n.is_matching(&neighbor_node))
                        .is_some()
                    {
                        open_set = open_set
                            .into_iter()
                            .map(|mut node| {
                                if node.is_matching(&neighbor_node) {
                                    node.est_cost = neighbor_node.est_cost;
                                }
                                node
                            })
                            .collect()
                    } else {
                        open_set.push(neighbor_node.clone());
                    }
                    parent.insert(neighbor_node, current.pos);
                }
            }
            // Costs(&costs, &open_set, &prev_dirs).print(self.num_rows, self.num_cols, h);
            // wait();
        }

        // reconstruct path
        let find_min_node = |pos: (Row, Col)| {
            costs
                .iter()
                .filter(|(node, cost)| node.pos == pos)
                .min_by_key(|(_, cost)| **cost)
        };

        let (goal_node, goal_cost) = find_min_node(goal).unwrap();
        let mut optimal_path = BTreeMap::new();
        optimal_path.insert(goal, goal_node.last_dir);
        let mut c = goal_node;
        while let Some(prev) = parent.get(c).copied() {
            if let Some(prev_node) = find_min_node(prev).map(|(node, _)| node) {
                optimal_path.insert(prev, prev_node.last_dir);
                c = prev_node;
            } else {
                break;
            }
        }

        // print puzzle with path
        for row in 0..self.num_rows {
            for col in 0..self.num_cols {
                if let Some(d) = optimal_path.get(&(row, col)) {
                    print!("{}", d);
                } else {
                    print!("{}", self[(row, col)] as char);
                }
            }
            println!();
        }
        return *goal_cost;
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

#[derive(Debug, Eq, Clone)]
struct Node {
    pos: (usize, usize),
    last_dir: Direction,
    moves_left: u8,
    est_cost: usize,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.last_dir == other.last_dir
            && self.moves_left == other.moves_left
        // no est cost
    }
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
        self.last_dir.hash(state);
        self.moves_left.hash(state);
        // no est_cost
    }
}
impl Node {
    fn is_matching(&self, neighbor_node: &Node) -> bool {
        self.pos == neighbor_node.pos
            && self.last_dir == neighbor_node.last_dir
            && self.moves_left == neighbor_node.moves_left
    }
}

impl std::cmp::PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = (self.est_cost, self.pos, self.moves_left, self.last_dir);
        let b = (other.est_cost, other.pos, other.moves_left, self.last_dir);
        b.cmp(&a)
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
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
struct Costs<'a>(
    &'a BTreeMap<(Row, Col), usize>,
    &'a BinaryHeap<Node>,
    &'a BTreeMap<(Row, Col), VecDeque<Direction>>,
);

impl<'a> Costs<'a> {
    fn print(&self, num_rows: usize, num_cols: usize, h: impl Fn((Row, Col)) -> usize) {
        let open_set = self.1;
        for r in 0..num_rows {
            for c in 0..num_cols {
                let pos = (r, c);
                let mut prev_dir = self
                    .2
                    .get(&pos)
                    .map(|dirs| {
                        let mut buf = String::new();
                        if let Some(d) = dirs.back() {
                            buf.push_str(&format!("{}", d));
                        }
                        buf
                    })
                    .unwrap_or("".into());
                prev_dir.push_str(&match open_set.iter().find(|x| x.pos == pos) {
                    Some(n) => format!("{}* ", n.est_cost),
                    None => match self.0.get(&pos) {
                        Some(c) => format!("{c} "),
                        None => format!("{}** ", h(pos)),
                    },
                });
                print!("{prev_dir:>7} ");
            }
            println!();
        }
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
