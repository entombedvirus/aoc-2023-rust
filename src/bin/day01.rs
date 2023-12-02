use anyhow::Result;
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    fn part_one_parse_number(line: &str) -> u32 {
        let first_digit = line.chars().find_map(|c| c.to_digit(10)).unwrap_or(0);
        let last_digit = line.chars().rev().find_map(|c| c.to_digit(10)).unwrap_or(0);
        first_digit * 10 + last_digit
    }

    Ok(input.lines().map(|l| part_one_parse_number(l)).sum())
}

const SPELLED_DIGITS: [&str; 10] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

fn part_two(input: &str) -> Result<u32> {
    Ok(input.lines().map(|l| part_two_parse_number(l)).sum())
}

fn part_two_parse_number(line: &str) -> u32 {
    let first_digit = parse_left_digit(line);
    let last_digit = parse_right_digit(line);
    first_digit * 10 + last_digit
}

fn parse_left_digit(line: &str) -> u32 {
    let numeric_digit = line
        .chars()
        .enumerate()
        .find_map(|(idx, c)| c.to_digit(10).map(|d| (idx, d)));

    let spelled_digit = SPELLED_DIGITS
        .iter()
        .enumerate()
        .filter_map(|(numeric_value, spelled_digit)| {
            line.find(spelled_digit)
                .map(|pos| (pos, numeric_value as u32))
        })
        .min_by_key(|(pos, _)| *pos);
    match (numeric_digit, spelled_digit) {
        (None, None) => 0,
        (None, Some((_, d))) => d,
        (Some((_, d)), None) => d,
        (Some((p1, d1)), Some((p2, d2))) => {
            if p1 < p2 {
                d1
            } else {
                d2
            }
        }
    }
}

fn parse_right_digit(line: &str) -> u32 {
    let numeric_digit = line
        .chars()
        .enumerate()
        .filter_map(|(idx, c)| c.to_digit(10).map(|d| (idx, d)))
        .last();
    let spelled_digit = SPELLED_DIGITS
        .iter()
        .enumerate()
        .filter_map(|(numeric_value, spelled_digit)| {
            line.rfind(spelled_digit)
                .map(|pos| (pos, numeric_value as u32))
        })
        .max_by_key(|(pos, _)| *pos);
    match (numeric_digit, spelled_digit) {
        (None, None) => 0,
        (None, Some((_, d))) => d,
        (Some((_, d)), None) => d,
        (Some((p1, d1)), Some((p2, d2))) => {
            if p1 > p2 {
                d1
            } else {
                d2
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PART_ONE_EXAMPLE_INPUT: &str = r#"1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet"#;
    const PART_TWO_EXAMPLE_INPUT: &str = r#"two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen"#;

    #[test]
    fn test_part_one() {
        assert_eq!(part_one(PART_ONE_EXAMPLE_INPUT).unwrap(), 142);
    }

    #[test]
    fn test_part_two() {
        assert_eq!(part_two(PART_TWO_EXAMPLE_INPUT).unwrap(), 281);
    }

    #[test]
    fn test_parse_left_digit() {
        assert_eq!(parse_left_digit("eightwothree"), 8);
        assert_eq!(parse_left_digit("abcone2threexyz"), 1);
        assert_eq!(parse_left_digit("4nineeightseven2"), 4);
        assert_eq!(parse_left_digit("9j"), 9);
    }

    #[test]
    fn test_parse_right_digit() {
        assert_eq!(parse_right_digit("eightwothree"), 3);
        assert_eq!(parse_right_digit("abcone2threexyz"), 3);
        assert_eq!(parse_right_digit("4nineeightseven2"), 2);
        assert_eq!(parse_right_digit("9j"), 9);
    }

    #[test]
    fn test_part_two_parsed_numbers() {
        let expected = [29, 83, 13, 24, 42, 14, 76];
        assert_eq!(
            PART_TWO_EXAMPLE_INPUT
                .lines()
                .map(part_two_parse_number)
                .collect::<Vec<_>>(),
            &expected
        );
    }

    #[test]
    fn test_part_two_parsed_numbers_edge_cases() {
        assert_eq!(part_two_parse_number("9j"), 99);
        assert_eq!(parse_left_digit("sixfconesix6three1sixsix"), 6);
        assert_eq!(parse_right_digit("sixfconesix6three1sixsix"), 6);
        assert_eq!(part_two_parse_number("sixfconesix6three1sixsix"), 66);
    }
}
