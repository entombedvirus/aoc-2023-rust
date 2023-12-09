#![feature(slice_group_by)]

use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{
    character::complete::{self, anychar, newline, space1},
    combinator::{map, map_res, opt},
    multi::{count, separated_list1},
    sequence::{separated_pair, terminated},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let mut hands = Hand::parse(input)?;
    hands.sort_unstable_by(Hand::ranking);
    Ok(hands
        .into_iter()
        .enumerate()
        .map(|(idx, h)| (idx + 1) * h.bid)
        .sum())
}

fn part_two(input: &str) -> Result<usize> {
    let mut hands = Hand::parse(input)?;
    hands.sort_unstable_by(Hand::ranking_with_jokers);
    Ok(hands
        .into_iter()
        .enumerate()
        .map(|(idx, h)| (idx + 1) * h.bid)
        .sum())
}

#[derive(Debug, PartialEq, Eq)]
struct Hand {
    cards: [Card; 5],
    bid: usize,
    hand_type: HandType,
}

impl Hand {
    fn parse(input: &str) -> Result<Vec<Self>> {
        let parse_cards = map(count(anychar, 5), |chars| {
            chars.into_iter().map(Card::from_char).collect()
        });
        let parse_hand = map_res(parse_cards, |cards: Vec<Card>| cards.try_into());
        let parse_game = map(
            separated_pair(parse_hand, space1, complete::u64),
            |(cards, bid)| {
                let hand_type = Self::compute_hand_type(&cards);
                Self {
                    cards,
                    bid: bid as usize,
                    hand_type,
                }
            },
        );
        let parser = terminated(separated_list1(newline, parse_game), opt(newline));
        must_parse(parser, input)
    }

    fn ranking_with_jokers(a: &Self, b: &Self) -> std::cmp::Ordering {
        match Self::compute_hand_type_with_jokers(&a.cards)
            .cmp(&Self::compute_hand_type_with_jokers(&b.cards))
        {
            std::cmp::Ordering::Equal => a
                .cards
                .iter()
                .zip(b.cards.iter())
                .find(|(ac, bc)| ac != bc)
                .map(|(ac, bc)| Card::compare_with_jokers(ac, bc))
                .unwrap_or(std::cmp::Ordering::Equal),
            ord => ord,
        }
    }

    fn compute_hand_type_with_jokers(cards: &[Card; 5]) -> HandType {
        let mut cards_without_jokers: Vec<_> = cards.iter().filter(|c| **c != Card::Jack).collect();
        cards_without_jokers.sort();

        let num_jokers = cards.len() - cards_without_jokers.len();
        if num_jokers == 5 {
            return HandType::FiveOfAKind;
        }

        let mut counts: Vec<_> = cards_without_jokers
            .group_by(|c1, c2| c1 == c2)
            .map(|g| g.len())
            .collect();
        counts.sort();
        counts.last_mut().map(|sl| *sl += num_jokers);
        Self::count_matcher(&counts)
    }

    fn ranking(a: &Self, b: &Self) -> std::cmp::Ordering {
        match a.hand_type.cmp(&b.hand_type) {
            std::cmp::Ordering::Equal => a
                .cards
                .iter()
                .zip(b.cards.iter())
                .find(|(ac, bc)| ac != bc)
                .map(|(ac, bc)| ac.cmp(bc))
                .unwrap_or(std::cmp::Ordering::Equal),
            ord => ord,
        }
    }

    fn compute_hand_type(cards: &[Card; 5]) -> HandType {
        let mut cards = *cards;
        cards.sort();
        let mut counts: Vec<_> = cards.group_by(|c1, c2| c1 == c2).map(|g| g.len()).collect();
        counts.sort();
        Self::count_matcher(&counts)
    }

    fn count_matcher(counts: &[usize]) -> HandType {
        use HandType::*;
        match counts {
            &[5] => FiveOfAKind,
            &[1, 4] => FourOfAKind,
            &[2, 3] => FullHouse,
            &[1, 1, 3] => ThreeOfAKind,
            &[1, 2, 2] => TwoPair,
            &[1, 1, 1, 2] => OnePair,
            &[1, 1, 1, 1, 1] => HighCard,
            unknown => unreachable!("unexpected card grouping: {unknown:?}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Card {
    fn from_char(ch: char) -> Self {
        use Card::*;
        match ch {
            '2' => Two,
            '3' => Three,
            '4' => Four,
            '5' => Five,
            '6' => Six,
            '7' => Seven,
            '8' => Eight,
            '9' => Nine,
            'T' => Ten,
            'J' => Jack,
            'Q' => Queen,
            'K' => King,
            'A' => Ace,
            unknown => panic!("Card::from_char unknown ch: {unknown}"),
        }
    }

    fn compare_with_jokers(a: &Self, b: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        use Card::*;
        match (a, b) {
            (&Jack, &Jack) => Equal,
            (&Jack, _) => Less,
            (_, &Jack) => Greater,
            (a, b) => a.cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Card::*;
    const INPUT: &str = r#"32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483"#;

    #[test]
    fn test_parse() -> Result<()> {
        let games = Hand::parse(INPUT)?;
        assert_eq!(
            &games[0],
            &Hand {
                cards: [Three, Two, Ten, Three, King],
                bid: 765,
                hand_type: HandType::OnePair,
            }
        );
        assert_eq!(
            &games[4],
            &Hand {
                cards: [Queen, Queen, Queen, Jack, Ace],
                bid: 483,
                hand_type: HandType::ThreeOfAKind,
            }
        );
        Ok(())
    }

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 6440);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 5905);
        Ok(())
    }
}
