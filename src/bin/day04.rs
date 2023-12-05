use std::collections::HashSet;

use anyhow::Result;
use aoc::runner;
use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::multispace1,
    combinator::map_res,
    multi::separated_list1,
    sequence::{delimited, tuple},
    Finish, IResult,
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let cards: Result<Vec<_>> = input.lines().map(Card::parse).collect();
    Ok(cards?.into_iter().map(|g| g.points()).sum())
}

fn part_two(input: &str) -> Result<usize> {
    let mut cards = input.lines().map(Card::parse).collect::<Result<Vec<_>>>()?;
    play_game(&mut cards);
    Ok(cards.into_iter().map(|c| c.num_copies).sum())
}

fn play_game(cards: &mut Vec<Card>) {
    for idx in 0..cards.len() {
        let card = &cards[idx];
        let num_copies_to_add = card.num_copies;
        let dup_start_idx = idx + 1;
        let dup_end_idx = dup_start_idx + card.num_winning();
        for c in &mut cards[dup_start_idx..dup_end_idx] {
            c.num_copies += num_copies_to_add;
        }
    }
}

#[derive(Debug)]
struct Card {
    game_no: usize,
    num_copies: usize,
    winning_numbers: HashSet<usize>,
    card_numbers: HashSet<usize>,
}

impl Card {
    fn parse(line: &str) -> Result<Self> {
        fn number(input: &str) -> IResult<&str, usize> {
            map_res(take_while1(|ch: char| ch.is_digit(10)), |num_str: &str| {
                num_str.parse::<usize>()
            })(input)
        }

        // Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
        let (rem, (_, _, game_no, _, _, winning_numbers, _, card_numbers)) = tuple((
            tag("Card"),
            multispace1,
            number,
            tag(":"),
            multispace1,
            separated_list1(multispace1, number),
            delimited(multispace1, tag("|"), multispace1),
            separated_list1(multispace1, number),
        ))(line)
        .finish()
        .map_err(|err| anyhow::format_err!("{}", err))?;
        anyhow::ensure!(
            rem.is_empty(),
            "expected line to be fully parsed. unparsed trailer: `{rem}`"
        );
        let winning_numbers: HashSet<usize> = winning_numbers.into_iter().collect();
        let card_numbers: HashSet<usize> = card_numbers.into_iter().collect();
        Ok(Self {
            game_no,
            num_copies: 1,
            winning_numbers,
            card_numbers,
        })
    }

    fn num_winning(&self) -> usize {
        self.winning_numbers
            .intersection(&self.card_numbers)
            .count()
    }

    fn points(&self) -> usize {
        match self.num_winning() {
            0 => 0,
            n => 2_usize.pow(n as u32 - 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 13);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 30);
        Ok(())
    }
}
