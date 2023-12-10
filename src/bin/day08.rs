use std::collections::BTreeMap;

use anyhow::{Context, Result};
use aoc::{must_parse, runner};
use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{self, alphanumeric1, newline},
    combinator::{map, opt},
    multi::{count, separated_list1},
    sequence::{delimited, separated_pair, terminated},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let map = Map::parse(input)?;

    let mut count = 0;
    let mut nodes = map.lookup("AAA");
    for ins in map.ins_iter() {
        count += 1;
        let next = match ins {
            Ins::Left => nodes.0,
            Ins::Right => nodes.1,
        };
        if next == "ZZZ" {
            break;
        }
        nodes = map.lookup(next);
    }
    Ok(count)
}

fn part_two(input: &str) -> Result<usize> {
    let map = Map::parse(input)?;
    let starting_nodes = map.network.keys().filter(|k| k.ends_with("A"));

    // NOTE: the path each starting node to the terminating node is cyclic. This means that
    // the point at which all starting nodes will arrive at a terminating node is the
    // lowest common multiple of the number of steps for each starting node.
    starting_nodes
        .map(|n| {
            map.nodes_iter(n)
                .take_while(|n| n.ends_with("Z") == false)
                .count()
                + 1 // + 1 to count the last node; take_while doesn't include the last node
        })
        .reduce(compute_lcm)
        .context("Unable to find lowest common denominator")
}

// See: https://en.wikipedia.org/wiki/Least_common_multiple
fn compute_lcm(a: usize, b: usize) -> usize {
    a * (b / compute_gcd_euclid(a, b))
}

// See: https://en.wikipedia.org/wiki/Euclidean_algorithm
fn compute_gcd_euclid(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp
    }
    a
}

#[derive(Debug)]
struct Map<'i> {
    instructions: &'i str,
    network: BTreeMap<&'i str, (&'i str, &'i str)>,
}

impl<'i> Map<'i> {
    fn parse(input: &'i str) -> Result<Self> {
        let parse_instructions = is_not("\n");
        let parse_comma_separated = separated_pair(alphanumeric1, tag(", "), alphanumeric1);
        let parse_network_tuple = delimited(
            complete::char('('),
            parse_comma_separated,
            complete::char(')'),
        );
        let parse_network_line = separated_pair(alphanumeric1, tag(" = "), parse_network_tuple);
        let parse_network = map(separated_list1(newline, parse_network_line), |x| {
            BTreeMap::from_iter(x)
        });
        let parser = map(
            terminated(
                separated_pair(parse_instructions, count(newline, 2), parse_network),
                opt(newline),
            ),
            |(instructions, network)| Self {
                instructions,
                network,
            },
        );
        must_parse(parser, input)
    }

    fn ins_iter(&'i self) -> impl Iterator<Item = Ins> + 'i {
        self.instructions
            .chars()
            .map(|ch| match ch {
                'L' => Ins::Left,
                'R' => Ins::Right,
                unknown => panic!("invalid instruction char: {unknown}"),
            })
            .cycle()
    }

    fn lookup(&'i self, node_name: &str) -> (&'i str, &'i str) {
        self.network[node_name]
    }

    fn nodes_iter(&'i self, starting_node: &'i str) -> impl Iterator<Item = &'i str> {
        let mut cur = starting_node;
        self.ins_iter().map(move |ins| {
            cur = match ins {
                Ins::Left => self.lookup(cur).0,
                Ins::Right => self.lookup(cur).1,
            };
            cur
        })
    }
}

#[derive(Debug)]
enum Ins {
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT1: &str = r#"RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)"#;

    const INPUT2: &str = r#"LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT1)?, 2);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT2)?, 6);
        Ok(())
    }
}
