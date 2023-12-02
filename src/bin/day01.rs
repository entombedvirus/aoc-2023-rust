fn main() {
    const PART_ONE_EXAMPLE_INPUT: &str = r#"1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet"#;
    let input = std::env::args()
        .nth(1)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or(PART_ONE_EXAMPLE_INPUT.to_string());
    println!("{}", part_one(input));
}

fn part_one(input: String) -> u32 {
    let parse_digit = |line: &str| {
        let first_digit = line
            .chars()
            .find_map(|x| x.to_digit(10))
            .expect("digit to present");
        let last_digit = line
            .chars()
            .rev()
            .find_map(|x| x.to_digit(10))
            .expect("digit to be present");
        first_digit * 10 + last_digit
    };

    input.lines().map(parse_digit).sum()
}
