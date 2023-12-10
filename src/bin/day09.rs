#![feature(iter_map_windows)]
use anyhow::Result;
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<i32> {
    let report = Report::parse(input)?;
    Ok(report
        .readings
        .into_iter()
        .map(|hist| hist.predict_next())
        .sum())
}

fn part_two(input: &str) -> Result<i32> {
    let report = Report::parse(input)?;
    Ok(report
        .readings
        .into_iter()
        .map(|hist| hist.predict_prev())
        .sum())
}

#[derive(Debug)]
struct History {
    values: Vec<i32>,
}

impl History {
    fn predict_next(&self) -> i32 {
        self.diff_iter()
            .filter_map(|diffs| diffs.last().copied())
            .sum()
    }

    fn diff_iter<'a>(&'a self) -> impl Iterator<Item = Vec<i32>> + 'a {
        std::iter::successors(Some(self.values.clone()), |values| {
            if values.iter().all(|v| *v == 0) {
                None
            } else {
                Some(values.iter().map_windows(|[x, y]| **y - **x).collect())
            }
        })
    }

    fn predict_prev(&self) -> i32 {
        enum Op {
            Add,
            Sub,
        }
        use Op::*;
        self.diff_iter()
            .filter_map(|diffs| diffs.first().copied())
            .fold((Add, 0), |(op, acc), x| match op {
                Add => (Sub, acc + x),
                Sub => (Add, acc - x),
            })
            .1
    }
}

#[derive(Debug)]
struct Report {
    readings: Vec<History>,
}

impl Report {
    fn parse(input: &str) -> Result<Self> {
        let readings: Vec<_> = input
            .lines()
            .map(|line| History {
                values: line
                    .split_whitespace()
                    .map(|v| v.parse().expect("number parsing failed"))
                    .collect(),
            })
            .collect();
        Ok(Self { readings })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 114);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 2);
        Ok(())
    }
}
