use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{
    bytes::complete::is_a, character::complete::newline, combinator::map, multi::separated_list1,
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let mut p = Puzzle::parse(input)?;
    p.slide_north();
    Ok(p.compute_score())
}

fn part_two(input: &str) -> Result<u32> {
    let mut p = Puzzle::parse(input)?;
    //                          
    // 0 1 2 3 4 5 6 7 8 9 0 1 |2| 3 4 5 6 7 8 9 ...
    // a b c a a a b b c a a a |b| b c a a a b b ...
    // mu = 3, lambda = 6
    //
    // predict element after 12 cycles.
    // idx_within_cycle = (12 - 3) % 6 = 3
    // idx_from_begin = mu + idx_within_cycle = 3 + 3 = 6
    let (mu, lambda) = detect_cycles(&p, Puzzle::tilt_cycle);

    let idx_within_cycle = (1_000_000_000 - mu) % lambda;
    let n = mu + idx_within_cycle;
    // to get to the nth entry, we need to call tilt_cycle n - 1 times
    for _ in 0..n {
        p = p.tilt_cycle();
    }
    Ok(p.compute_score())
}

// See: Brent's algorithm
// (https://en.m.wikipedia.org/wiki/Cycle_detection#Floyd's_tortoise_and_hare)
fn detect_cycles(x0: &Puzzle, mut f: impl FnMut(&Puzzle) -> Puzzle) -> (usize, usize) {
    // main phase: search successive powers of two
    let mut power = 1;
    let mut lambda = 1;
    let mut tortoise = x0.clone();
    let mut hare = f(x0);
    while tortoise != hare {
        if power == lambda {
            tortoise = hare.clone();
            power *= 2;
            lambda = 0;
        }
        hare = f(&hare);
        lambda += 1;
    }

    // Find the position of the first repetition of length λ
    tortoise = x0.clone();
    hare = x0.clone();
    for _ in 0..lambda {
        hare = f(&hare);
    }

    // The distance between the hare and tortoise is now λ.

    // Next, the hare and tortoise move at same speed until they agree
    let mut mu = 0;
    while tortoise != hare {
        tortoise = f(&tortoise);
        hare = f(&hare);
        mu += 1;
    }
    (mu, lambda)
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Puzzle {
    rows: Vec<String>,
}

impl Puzzle {
    fn parse(input: &str) -> Result<Self> {
        let parse_line = map(is_a("O.#"), |s: &str| s.to_string());
        let parse_lines = separated_list1(newline, parse_line);
        let parser = map(parse_lines, |rows| Self { rows });
        must_parse(parser, input)
    }

    fn num_cols(&self) -> usize {
        self.rows.first().map(|r| r.len()).unwrap_or(0)
    }

    fn tilt_cycle(&self) -> Self {
        let mut clone = self.clone();
        clone.slide_north();
        clone.slide_west();
        clone.slide_south();
        clone.slide_east();
        clone
    }

    fn slide_south(&mut self) {
        for col_idx in 0..self.num_cols() {
            let mut slot_idx = None;
            for row_idx in (0..self.rows.len()).rev() {
                let ch = self.rows[row_idx].as_bytes()[col_idx] as char;
                match ch {
                    '.' => {
                        slot_idx.get_or_insert(row_idx);
                    }
                    'O' => match slot_idx.as_mut() {
                        Some(dest_idx) => {
                            self.rows[*dest_idx].replace_range(col_idx..col_idx + 1, "O");
                            self.rows[row_idx].replace_range(col_idx..col_idx + 1, ".");
                            *dest_idx -= 1;
                        }
                        None => continue,
                    },
                    '#' => {
                        slot_idx.take();
                    }
                    unknown => unreachable!("unknown char: {unknown}"),
                };
            }
        }
    }

    fn slide_north(&mut self) {
        for col_idx in 0..self.num_cols() {
            let mut slot_idx = None;
            for row_idx in 0..self.rows.len() {
                let ch = self.rows[row_idx].as_bytes()[col_idx] as char;
                match ch {
                    '.' => {
                        slot_idx.get_or_insert(row_idx);
                    }
                    'O' => match slot_idx.as_mut() {
                        Some(dest_idx) => {
                            self.rows[*dest_idx].replace_range(col_idx..col_idx + 1, "O");
                            self.rows[row_idx].replace_range(col_idx..col_idx + 1, ".");
                            *dest_idx += 1;
                        }
                        None => continue,
                    },
                    '#' => {
                        slot_idx.take();
                    }
                    unknown => unreachable!("unknown char: {unknown}"),
                };
            }
        }
    }

    fn slide_west(&mut self) {
        for row_idx in 0..self.rows.len() {
            let mut slot_idx = None;
            for col_idx in 0..self.num_cols() {
                let ch = self.rows[row_idx].as_bytes()[col_idx] as char;
                match ch {
                    '.' => {
                        slot_idx.get_or_insert(col_idx);
                    }
                    'O' => match slot_idx.as_mut() {
                        Some(dest_idx) => {
                            self.rows[row_idx].replace_range(*dest_idx..*dest_idx + 1, "O");
                            self.rows[row_idx].replace_range(col_idx..col_idx + 1, ".");
                            *dest_idx += 1;
                        }
                        None => continue,
                    },
                    '#' => {
                        slot_idx.take();
                    }
                    unknown => unreachable!("unknown char: {unknown}"),
                };
            }
        }
    }

    fn slide_east(&mut self) {
        for row_idx in 0..self.rows.len() {
            let mut slot_idx = None;
            for col_idx in (0..self.num_cols()).rev() {
                let ch = self.rows[row_idx].as_bytes()[col_idx] as char;
                match ch {
                    '.' => {
                        slot_idx.get_or_insert(col_idx);
                    }
                    'O' => match slot_idx.as_mut() {
                        Some(dest_idx) => {
                            self.rows[row_idx].replace_range(*dest_idx..*dest_idx + 1, "O");
                            self.rows[row_idx].replace_range(col_idx..col_idx + 1, ".");
                            *dest_idx -= 1;
                        }
                        None => continue,
                    },
                    '#' => {
                        slot_idx.take();
                    }
                    unknown => unreachable!("unknown char: {unknown}"),
                };
            }
        }
    }

    // for each column, sum the distance from the "southern" edge
    fn compute_score(&self) -> u32 {
        let total_len = self.num_cols() as u32;
        (0..self.num_cols())
            .flat_map(|col_num| {
                self.rows
                    .iter()
                    .map(move |r| r.as_bytes()[col_num])
                    .enumerate()
                    .filter_map(|(idx, ch)| (ch == b'O').then_some(total_len - idx as u32))
            })
            .sum()
    }
}

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in &self.rows {
            writeln!(f, "{r}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#...."#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 136);
        Ok(())
    }

    #[test]
    fn test_cycle() -> Result<()> {
        let p = Puzzle::parse(INPUT)?;
        let expected = vec![
            r#".....#....
....#...O#
...OO##...
.OO#......
.....OOO#.
.O#...O#.#
....O#....
......OOOO
#...O###..
#..OO#...."#,
            r#".....#....
....#...O#
.....##...
..O#......
.....OOO#.
.O#...O#.#
....O#...O
.......OOO
#..OO###..
#.OOO#...O"#,
            r#".....#....
....#...O#
.....##...
..O#......
.....OOO#.
.O#...O#.#
....O#...O
.......OOO
#...O###.O
#.OOO#...O"#,
        ];

        let p = p.tilt_cycle();
        assert_eq!(p, Puzzle::parse(expected[0])?);
        let p = p.tilt_cycle();
        assert_eq!(p, Puzzle::parse(expected[1])?);
        let p = p.tilt_cycle();
        assert_eq!(p, Puzzle::parse(expected[2])?);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 64);
        Ok(())
    }
}
