use anyhow::Result;
use aoc::runner;
use nom::{
    bytes::complete::tag,
    character::complete::{self, multispace1, newline, space1},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{preceded, separated_pair, terminated, tuple},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u64> {
    let sheet = Sheet::parse(input)?;
    Ok(sheet
        .races()
        .into_iter()
        .map(|r| r.num_ways_derive())
        .product())
}

fn part_two(input: &str) -> Result<u64> {
    let sheet = Sheet::parse(input)?;
    Ok(sheet.concatenated_race().num_ways_derive())
}

#[derive(Debug)]
struct Sheet {
    times: Vec<u64>,
    distances: Vec<u64>,
}

impl Sheet {
    fn races<'a>(&'a self) -> impl Iterator<Item = Race> + 'a {
        self.times
            .iter()
            .zip(self.distances.iter())
            .map(|(&duration, &distance)| Race {
                duration,
                record_distance: distance,
            })
    }

    fn parse(input: &str) -> Result<Self> {
        let parse_distances = preceded(
            tuple((tag("Distance:"), multispace1)),
            separated_list1(space1, complete::u64),
        );
        let parse_times = preceded(
            tuple((tag("Time:"), multispace1)),
            separated_list1(space1, complete::u64),
        );
        let mut parser = map(
            terminated(
                separated_pair(parse_times, newline, parse_distances),
                opt(newline),
            ),
            |(times, distances)| Self { times, distances },
        );
        let (rem, sheet) = parser(input)
            .map_err(|err: nom::Err<nom::error::Error<&str>>| anyhow::format_err!("{}", err))?;
        anyhow::ensure!(rem.is_empty(), "parsing terminated early: {rem}");
        Ok(sheet)
    }

    fn concatenated_race(&self) -> Race {
        let duration: String = self.times.iter().map(|&t| t.to_string()).collect();
        let record_distance: String = self.distances.iter().map(|&t| t.to_string()).collect();
        Race {
            duration: duration.parse().expect("duration parse failed"),
            record_distance: record_distance
                .parse()
                .expect("record_distance parse failed"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Race {
    duration: u64,
    record_distance: u64,
}

impl Race {
    // dist = (total_duration - charge_ms) * charge_ms
    //
    // we can find out at which charge_ms value we hit the record distance using formula above:
    //
    // r = (total_duration - charge_ms) * charge_ms
    // => r = td * c - c^2
    // => 0 = -1c^2 + td*c - r
    // => c = (-td +- sqrt(td^2 - 4*-1*-r)) / 2 * -1
    // => c = (-td +- sqrt(td*td - 4r)) / -2
    //
    // since we know at what values (there's always two) of charge_ms the current record is
    // achieved, we can compute the number of ways to win by counting the number of whole number in
    // that between intervals.
    fn num_ways_derive(&self) -> u64 {
        let td = self.duration as f64;
        let r = self.record_distance as f64;
        let sqrt = (td * td - 4. * r).sqrt();
        let mut sols = ((-td + sqrt) / -2., (-td - sqrt) / -2.);
        if sols.0 > sols.1 {
            sols = (sols.1, sols.0);
        }
        let mut whole_nums = sols.1 - sols.0.floor();
        if whole_nums.floor() == whole_nums {
            whole_nums -= 1.;
        }
        whole_nums as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"Time:      7  15   30
Distance:  9  40  200"#;

    #[test]
    fn test_sheet_parse() -> Result<()> {
        let sheet = Sheet::parse(INPUT)?;
        assert_eq!(sheet.times, vec![7, 15, 30]);
        assert_eq!(sheet.distances, vec![9, 40, 200]);
        Ok(())
    }

    #[test]
    fn test_concatenated_race() -> Result<()> {
        let sheet = Sheet::parse(INPUT)?;
        assert_eq!(
            sheet.concatenated_race(),
            Race {
                duration: 71530,
                record_distance: 940200
            }
        );
        Ok(())
    }

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 288);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 71503);
        Ok(())
    }
}
