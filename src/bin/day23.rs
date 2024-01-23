use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use aoc::bit_set::BitSet;
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u16> {
    let p = Puzzle::parse(input)?;
    p.longest_path()
}

fn part_two(input: &str) -> Result<u16> {
    let mut p = Puzzle::parse(input)?;

    // treat all slides as normal tiles
    for tile in &mut p.tiles {
        match *tile {
            b'>' | b'<' | b'^' | b'v' => *tile = b'.',
            _ => (),
        }
    }

    p.longest_path()
}

type Pos = (isize, isize);

#[derive(Debug)]
struct CostLut(Vec<Vec<u16>>);

impl CostLut {
    unsafe fn get<'a>(
        &'a self,
        from_idx: usize,
        indices: BitSet,
    ) -> impl Iterator<Item = u16> + 'a {
        let costs = self.0.get_unchecked(from_idx);
        indices
            .into_iter()
            .map(|idx| *costs.get_unchecked(idx.get()))
    }
}

#[derive(Debug, Default)]
struct Graph {
    neighbors: BTreeMap<Pos, BTreeSet<Pos>>,
    edges: BTreeMap<(Pos, Pos), u16>,
}

impl Graph {
    fn add_directed_edge(&mut self, n1: Pos, n2: Pos, cost: u16) {
        self.edges.entry((n1, n2)).or_insert(cost);
        self.neighbors.entry(n1).or_default().insert(n2);
    }

    fn get_sorted_nodes(&self) -> Vec<Pos> {
        self.edges
            .keys()
            .cloned()
            .flat_map(|(n1, n2)| [n1, n2])
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn get_neighbor_bitsets(&self, nodes: &[Pos]) -> Result<Vec<BitSet>> {
        anyhow::ensure!(
            nodes.len() <= 64,
            "cannot only handle graphs with <= 64 nodes"
        );
        let build_bitset = |node| {
            let Some(neighbors) = self.neighbors.get(node) else {
                return BitSet::new();
            };
            let find_position = |neighbor| {
                nodes
                    .iter()
                    .position(|x| x == neighbor)
                    .expect("all neighbors are expected to be in nodes")
            };
            BitSet::from_iter(neighbors.iter().map(find_position))
        };
        Ok(nodes.iter().map(build_bitset).collect())
    }

    fn get_costs_lookup_table(&self, nodes: &[Pos]) -> CostLut {
        let mut lut = vec![vec![0u16; nodes.len()]; nodes.len()];
        for ((from, to), cost) in &self.edges {
            let from_idx = nodes
                .iter()
                .position(|n| n == from)
                .expect("from node not found");
            let to_idx = nodes
                .iter()
                .position(|n| n == to)
                .expect("to node not found");
            lut[from_idx][to_idx] = *cost;
        }
        CostLut(lut)
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
        let mut push_if_valid = |new_pos| {
            match self.get_tile(new_pos) {
                Some(b'<') if cur_pos != (new_pos.0, new_pos.1 - 1) => npos.push(new_pos),
                Some(b'>') if cur_pos != (new_pos.0, new_pos.1 + 1) => npos.push(new_pos),
                Some(b'^') if cur_pos != (new_pos.0 - 1, new_pos.1) => npos.push(new_pos),
                Some(b'v') if cur_pos != (new_pos.0 + 1, new_pos.1) => npos.push(new_pos),
                Some(b'.') => npos.push(new_pos),
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

    fn longest_path(&self) -> Result<u16> {
        let (nodes, neighbors, costs) = {
            let graph = self.as_graph();

            let nodes = graph.get_sorted_nodes();
            let neighbors = graph.get_neighbor_bitsets(&nodes)?;
            let costs = graph.get_costs_lookup_table(&nodes);
            (nodes, neighbors, costs)
        };

        // nodes are sorted by row number first, hence start and finish end up as being first and last
        // nodes respectively
        let start_idx = 0;
        let finish_idx = nodes.len() - 1;

        let mut longest_path = None;

        let mut q = Vec::with_capacity(nodes.len());
        q.push((start_idx, BitSet::new(), 0u16));

        // DFS to see all paths to finish, keeping track of max cost
        while let Some((from_node_idx, mut seen, cost)) = q.pop() {
            seen.set(from_node_idx);
            if from_node_idx == finish_idx {
                longest_path = std::cmp::max(longest_path, Some(cost));
                continue;
            }
            let neighbors = neighbors[from_node_idx].difference(seen);
            // SAFETY: from_node_idx and neighbors indexes are guaranteed to be less than or equal to
            // 64 due to the earlier assert while constructing BitSet
            let costs = unsafe { costs.get(from_node_idx, neighbors) };
            for (ncost, neighbor_idx) in costs.zip(neighbors) {
                q.push((neighbor_idx.get(), seen, cost + ncost));
            }
        }

        longest_path.ok_or(anyhow::format_err!("path to finish not found"))
    }

    fn as_graph(&self) -> Graph {
        let nodes: Vec<_> = (0..self.num_rows)
            .flat_map(|r| (0..self.num_cols).map(move |c| (r as isize, c as isize)))
            .filter(|(r, c)| {
                let mut valid_npos = 0;
                if self.get_tile((*r, *c)) != Some(b'#') {
                    for (dr, dc) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                        if let Some(tile) = self.get_tile((r + dr, c + dc)) {
                            if tile != b'#' {
                                valid_npos += 1;
                            }
                        }
                    }
                }
                // any position that 1 neighbor is the start and finish tile
                // 3 or 4 valid nighbors are junctions where multiple paths meet and therefore are
                //   nodes in our graph
                valid_npos > 0 && valid_npos != 2
            })
            .collect();

        let find_end_node = |start_node, mut cur_node| {
            let mut cost = 1_u16;
            let mut prev_node = start_node;
            while !nodes.contains(&cur_node) {
                let next_node = self
                    .get_neighbor_pos(cur_node)
                    .into_iter()
                    .find(|&npos| npos != prev_node)?;
                prev_node = cur_node;
                cur_node = next_node;
                cost += 1;
            }
            Some((cur_node, cost))
        };

        let mut graph = Graph::default();
        for start_node in &nodes {
            for cur_node in self.get_neighbor_pos(*start_node) {
                if let Some((end_node, cost)) = find_end_node(*start_node, cur_node) {
                    graph.add_directed_edge(*start_node, end_node, cost);
                }
            }
        }
        graph
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

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 154);
        Ok(())
    }
}
