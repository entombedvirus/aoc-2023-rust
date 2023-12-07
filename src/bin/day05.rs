use anyhow::Result;
use aoc::runner;
use nom::{
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, multispace1},
    combinator::{map, map_res, opt},
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let alm = Almanac::parse(input)?;
    Ok(alm
        .seeds
        .iter()
        .map(|seed| alm.location(*seed))
        .min()
        .expect("could not find min"))
}

fn part_two(_input: &str) -> Result<usize> {
    todo!()
}

type Seed = usize;
type Loc = usize;

#[derive(Debug)]
struct MappingRange {
    dest_start: usize,
    src_start: usize,
    len: usize,
}

impl MappingRange {
    fn lookup(&self, index: usize) -> Option<usize> {
        if index >= self.src_start && index < self.src_start + self.len {
            let delta = index - self.src_start;
            Some(self.dest_start + delta)
        } else {
            None
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Mapping {
    name: String,
    ranges: Vec<MappingRange>,
}

impl Mapping {
    fn lookup(&self, index: usize) -> usize {
        self.ranges
            .iter()
            .find_map(|r| r.lookup(index))
            .unwrap_or(index)
    }
}

#[derive(Debug)]
struct Almanac {
    seeds: Vec<Seed>,
    mappings: Vec<Mapping>,
}

impl Almanac {
    fn parse(input: &str) -> Result<Self> {
        fn number(input: &str) -> IResult<&str, usize> {
            map_res(take_while1(|ch: char| ch.is_digit(10)), |num_str: &str| {
                num_str.parse::<usize>()
            })(input)
        }
        let parse_seeds = separated_list1(char(' '), number);
        let parse_range = map(
            tuple((number, char(' '), number, char(' '), number)),
            |(dest_start, _, src_start, _, len)| MappingRange {
                dest_start,
                src_start,
                len,
            },
        );
        let parse_ranges = separated_list1(char('\n'), parse_range);
        let parse_mapping = map(
            tuple((is_not(" "), tag(" map:\n"), parse_ranges)),
            |(name, _, ranges)| Mapping {
                name: name.to_string(),
                ranges,
            },
        );
        let parse_mappings = separated_list1(tag("\n\n"), parse_mapping);
        let mut parser = map(
            tuple((
                tag("seeds: "),
                parse_seeds,
                multispace1,
                parse_mappings,
                opt(char('\n')),
            )),
            |(_, seeds, _, mappings, _)| Self { seeds, mappings },
        );

        let (rem, alm) = parser(input).map_err(|err| anyhow::format_err!("{}", err))?;
        anyhow::ensure!(
            rem.is_empty(),
            "failed to parse input completely. rem: {rem:?}"
        );
        Ok(alm)
    }

    fn location(&self, seed: Seed) -> Loc {
        self.mappings.iter().fold(seed, |acc, m| m.lookup(acc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4"#;

    #[test]
    fn test_parse() -> Result<()> {
        Almanac::parse(INPUT).map(|_| ())
    }

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 35);
        Ok(())
    }
}
