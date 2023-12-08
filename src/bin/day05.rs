#![feature(btree_cursors)]

use std::{
    collections::BTreeMap,
    fmt::Formatter,
    ops::{Bound, Range},
};

use anyhow::{Context, Result};
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

fn part_two(input: &str) -> Result<usize> {
    let alm = Almanac::parse(input)?;
    let input = NonOverlappingRanges::from_seeds(&alm.seeds);
    Ok(input
        .ranges
        .into_iter()
        .map(|(start, end)| alm.min_location_for(start..end))
        .min()
        .context("minimum location not found")?)
}

type Seed = usize;
type Loc = usize;

#[derive(Debug)]
struct MappingRange {
    dest_start: usize,
    src_start: usize,
    len: usize,
}

impl std::fmt::Display for MappingRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "MappingRange({src_range:?} -> {dest_range:?})",
            src_range = self.src_start..self.src_start + self.len,
            dest_range = self.dest_start..self.dest_start + self.len
        )
    }
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

    fn lookup_range(
        &self,
        input: Range<usize>,
    ) -> (
        Option<Range<usize>>,
        Option<Range<usize>>,
        Option<Range<usize>>,
    ) {
        let mut prefix = None;
        let mut middle = None;
        let mut suffix = None;
        let src_range = self.src_start..self.src_start + self.len;
        if input.end <= src_range.start {
            prefix = Some(input);
        } else if input.start >= src_range.end {
            suffix = Some(input);
        } else if input.start < self.src_start && input.end <= src_range.end {
            let overlap = input.end - self.src_start;
            prefix = Some(input.start..self.src_start);
            middle = Some(self.dest_start..self.dest_start + overlap);
        } else if input.start >= src_range.start && input.end <= src_range.end {
            let offset = input.start - self.src_start;
            let overlap = input.end - input.start;
            middle = Some(self.dest_start + offset..self.dest_start + offset + overlap);
        } else if input.start >= src_range.start
            && input.start < src_range.end
            && input.end >= src_range.end
        {
            let offset = input.start - self.src_start;
            let overlap = src_range.end - input.start;
            middle = Some(self.dest_start + offset..self.dest_start + offset + overlap);
            suffix = Some(input.start + overlap..input.end);
        } else if input.start < src_range.start && input.end >= src_range.end {
            prefix = Some(input.start..self.src_start);
            middle = Some(self.dest_start..self.dest_start + self.len);
            suffix = Some(src_range.end..input.end);
        } else {
            unreachable!()
        }
        (prefix, middle, suffix)
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Mapping {
    name: String,
    ranges: BTreeMap<usize, MappingRange>,
}

impl std::fmt::Display for Mapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{name}{ranges:?}",
            name = self.name,
            ranges = self.ranges
        )
    }
}
impl Mapping {
    fn lookup(&self, index: usize) -> usize {
        self.ranges
            .iter()
            .find_map(|(_, r)| r.lookup(index))
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
                ranges: ranges.into_iter().map(|x| (x.src_start, x)).collect(),
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

    fn min_location_for(&self, seed_range: Range<Seed>) -> Loc {
        let locs = self
            .mappings
            .iter()
            .fold(NonOverlappingRanges::single(seed_range), |inputs, m| {
                inputs.apply_mapping(m)
            });
        locs.ranges
            .keys()
            .next()
            .copied()
            .expect("min_locatation_for failed to find location")
    }
}

#[derive(Debug)]
struct NonOverlappingRanges {
    /// key is that start of the range, value is the end of the range, non-inclusive ex: the range
    /// 5..17 will be stored with a key of 5 and value of 17
    ranges: BTreeMap<usize, usize>,
}

impl NonOverlappingRanges {
    fn new(mut ranges: BTreeMap<usize, usize>) -> Self {
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

    fn from_seeds(seed_ranges: &[usize]) -> Self {
        let ranges: BTreeMap<usize, usize> = seed_ranges
            .chunks_exact(2)
            .map(|sl| (sl[0], sl[1]))
            .map(|(start, len)| (start, start + len))
            .collect();
        Self::new(ranges)
    }

    fn apply_mapping(&self, mapping: &Mapping) -> Self {
        let do_lookup = |mut input_range: Range<usize>| {
            let mut mapped_result = Vec::new();
            for (_, mr) in mapping.ranges.iter() {
                let (prefix, middle, suffix) = mr.lookup_range(input_range.clone());
                prefix.map(|p| mapped_result.push((p.start, p.end)));
                middle.map(|p| mapped_result.push((p.start, p.end)));
                if let Some(r) = suffix {
                    input_range = r.start..r.end;
                    continue;
                } else {
                    input_range = 0..0;
                    break;
                }
            }
            if input_range.len() != 0 {
                mapped_result.push((input_range.start, input_range.end));
            }

            mapped_result
        };
        self.ranges
            .iter()
            .flat_map(|(istart, iend)| do_lookup(*istart..*iend).into_iter())
            .collect()
    }

    fn single(seed_range: Range<usize>) -> Self {
        Self::from_iter(std::iter::once((seed_range.start, seed_range.end)))
    }
}

impl FromIterator<(usize, usize)> for NonOverlappingRanges {
    fn from_iter<T: IntoIterator<Item = (usize, usize)>>(iter: T) -> Self {
        let mut ranges = BTreeMap::new();
        for (istart, iend) in iter {
            let slot = ranges.entry(istart).or_insert(iend);
            *slot = iend.max(*slot);
        }
        Self::new(ranges)
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
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 46);
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
        assert_eq!(lookup.lookup_range(1..5), (Some(1..5), None, None));
        assert_eq!(lookup.lookup_range(15..20), (None, None, Some(15..20)));

        assert_eq!(
            lookup.lookup_range(1..6),
            (Some(1..5), Some(100..101), None)
        );
        assert_eq!(
            lookup.lookup_range(1..15),
            (Some(1..5), Some(100..110), None)
        );

        assert_eq!(lookup.lookup_range(5..15), (None, Some(100..110), None));
        assert_eq!(lookup.lookup_range(6..14), (None, Some(101..109), None));

        assert_eq!(
            lookup.lookup_range(14..16),
            (None, Some(109..110), Some(15..16))
        );
        assert_eq!(
            lookup.lookup_range(13..17),
            (None, Some(108..110), Some(15..17))
        );

        assert_eq!(
            lookup.lookup_range(1..17),
            (Some(1..5), Some(100..110), Some(15..17))
        );
    }
}
