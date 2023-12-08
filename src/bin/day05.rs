#![feature(btree_cursors)]

use std::{
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap,
    },
    ops::{Bound, Range},
};

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

    fn lookup_range(&self, input: Range<usize>) -> Vec<(usize, usize)> {
        let src_range = self.src_start..self.src_start + self.len;
        if input.end <= src_range.start || input.start >= src_range.end {
            vec![(input.start, input.end)]
        } else if input.start < self.src_start && input.end <= src_range.end {
            let overlap = input.end - self.src_start;
            vec![
                (input.start, self.src_start),
                (self.dest_start, self.dest_start + overlap),
            ]
        } else if input.start >= src_range.start && input.end <= src_range.end {
            let offset = input.start - self.src_start;
            let overlap = input.end - input.start;
            vec![(self.dest_start + offset, self.dest_start + offset + overlap)]
        } else if input.start >= src_range.start
            && input.start < src_range.end
            && input.end >= src_range.end
        {
            let offset = input.start - self.src_start;
            let overlap = src_range.end - input.start;
            vec![
                (self.dest_start + offset, self.dest_start + offset + overlap),
                (input.start + overlap, input.end),
            ]
        } else if input.start < src_range.start && input.end >= src_range.end {
            vec![
                (input.start, self.src_start),
                (self.dest_start, self.dest_start + self.len),
                (src_range.end, input.end),
            ]
        } else {
            unreachable!()
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

#[derive(Debug)]
struct NonOverlappingRanges {
    /// key is that start of the range, value is the end of the range, non-inclusive ex: the range
    /// 5..17 will be stored with a key of 5 and value of 17
    ranges: BTreeMap<usize, usize>,
}

impl NonOverlappingRanges {
    fn from_seeds(seed_ranges: &[usize]) -> Self {
        let mut ranges: BTreeMap<usize, usize> = seed_ranges
            .chunks_exact(2)
            .map(|sl| (sl[0], sl[1]))
            .map(|(start, len)| (start, start + len))
            .collect();
        let mut cur = ranges.lower_bound_mut(Bound::Unbounded);
        while let Some(current_range) = cur.key_value_mut().map(|(cstart, cend)| *cstart..*cend) {
            // cursor is guaranteed to be less than nstart due to btree ordering
            let Some(next_range) = cur.peek_next().map(|(nstart, nend)| *nstart..*nend) else {
                break;
            };
            // 3 possibilities:
            if current_range.contains(&next_range.start) && current_range.contains(&next_range.end)
            {
                // 1. next range is inside cur range and we can delete next_range
                cur.move_next();
                cur.remove_current();
            } else if current_range.contains(&next_range.start)
                && !current_range.contains(&next_range.end)
            {
                // 2. next range overlap cur range and cur range needs to be extended
                cur.value_mut().map(|v| *v = next_range.end);
                cur.move_next();
                cur.remove_current();
            } else {
                // 3. next range does not overlap
                cur.move_next();
            }
        }

        Self { ranges }
    }

    fn apply_mapping(&self, mapping: &Mapping) -> Self {
        let lookup: BTreeMap<usize, &MappingRange> =
            mapping.ranges.iter().map(|x| (x.src_start, x)).collect();
        let mapped_ranges: BTreeMap<usize, usize> = self
            .ranges
            .iter()
            .flat_map(|(istart, iend)| {
                let cursor = lookup.upper_bound(Bound::Included(istart));
                match cursor.value() {
                    Some(mr) => mr.lookup_range(*istart..*iend).into_iter(),
                    None => vec![(*istart, *iend)].into_iter(),
                }
            })
            .collect();
        Self {
            ranges: mapped_ranges,
        }
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

    #[test]
    fn test_non_overlapping_ranges() {
        let mut rs = NonOverlappingRanges::from_seeds(&vec![1, 10, 15, 5]);
        assert_eq!(rs.ranges.pop_first(), Some((1, 11)));
        assert_eq!(rs.ranges.pop_first(), Some((15, 20)));
        assert_eq!(rs.ranges.pop_first(), None);

        let mut rs = NonOverlappingRanges::from_seeds(&vec![1, 10, 1, 11]);
        assert_eq!(rs.ranges.pop_first(), Some((1, 12)));
        assert_eq!(rs.ranges.pop_first(), None);

        let mut rs = NonOverlappingRanges::from_seeds(&vec![1, 10, 2, 10]);
        assert_eq!(rs.ranges.pop_first(), Some((1, 12)));
        assert_eq!(rs.ranges.pop_first(), None);

        let mut rs = NonOverlappingRanges::from_seeds(&vec![1, 10, 2, 3]);
        assert_eq!(rs.ranges.pop_first(), Some((1, 11)));

        let mut rs = NonOverlappingRanges::from_seeds(&vec![2, 3, 1, 10]);
        assert_eq!(rs.ranges.pop_first(), Some((1, 11)));
        assert_eq!(rs.ranges.pop_first(), None);
        assert_eq!(rs.ranges.pop_first(), None);
    }

    #[test]
    fn test_range_lookup() {
        let lookup = MappingRange {
            dest_start: 100,
            src_start: 5,
            len: 10,
        };
        assert_eq!(lookup.lookup_range(1..5), vec![(1, 5)]);
        assert_eq!(lookup.lookup_range(15..20), vec![(15, 20)]);

        assert_eq!(lookup.lookup_range(1..6), vec![(1, 5), (100, 101)]);
        assert_eq!(lookup.lookup_range(1..15), vec![(1, 5), (100, 110)]);

        assert_eq!(lookup.lookup_range(5..15), vec![(100, 110)]);
        assert_eq!(lookup.lookup_range(6..14), vec![(101, 109)]);

        assert_eq!(lookup.lookup_range(14..16), vec![(109, 110), (15, 16)]);
        assert_eq!(lookup.lookup_range(13..17), vec![(108, 110), (15, 17)]);

        assert_eq!(
            lookup.lookup_range(1..17),
            vec![(1, 5), (100, 110), (15, 17)]
        );
    }
}
