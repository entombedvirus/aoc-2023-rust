use anyhow::Result;
use aoc::runner;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{multispace0, multispace1},
    combinator::map_res,
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

fn main() -> Result<()> {
    println!("heya");
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let q = CubeSet {
        red: 12,
        green: 13,
        blue: 14,
    };
    let games: Result<Vec<_>> = input.lines().map(Game::parse).collect();
    Ok(games?
        .into_iter()
        .filter_map(|g| g.is_possible(&q).then(|| g.game_id))
        .sum())
}

fn part_two(input: &str) -> Result<usize> {
    let games: Result<Vec<_>> = input.lines().map(Game::parse).collect();
    Ok(games?.into_iter().map(|g| g.minimum_set().power()).sum())
}

#[derive(PartialEq, Debug)]
enum Cubes {
    Red(usize),
    Green(usize),
    Blue(usize),
}

impl Cubes {
    // 1 blue
    // 2 green
    fn parse(input: &str) -> IResult<&str, Self> {
        use Cubes::*;
        let (rem, (_, n, _, color)) = tuple((
            multispace0,
            parse_number,
            multispace1,
            alt((tag("red"), tag("green"), tag("blue"))),
        ))(input)?;
        let cubes = match color {
            "red" => Red(n),
            "green" => Green(n),
            "blue" => Blue(n),
            unknown_color => panic!("unknown color: {unknown_color}"),
        };
        Ok((rem, cubes))
    }
}

#[derive(PartialEq, Debug, Default)]
struct CubeSet {
    red: usize,
    green: usize,
    blue: usize,
}

impl CubeSet {
    // 8 green, 6 blue, 20 red
    fn parse(input: &str) -> IResult<&str, Self> {
        let (rem, cubes) = separated_list1(tag(","), Cubes::parse)(input)?;
        let mut cube_set = Self {
            red: 0,
            green: 0,
            blue: 0,
        };
        for cube in cubes {
            match cube {
                Cubes::Red(n) => cube_set.red = n,
                Cubes::Green(n) => cube_set.green = n,
                Cubes::Blue(n) => cube_set.blue = n,
            }
        }
        Ok((rem, cube_set))
    }

    fn is_possible(&self, other: &Self) -> bool {
        self.red <= other.red && self.green <= other.green && self.blue <= other.blue
    }

    fn power(&self) -> usize {
        [self.red, self.green, self.blue]
            .into_iter()
            .filter(|n| *n > 0)
            .product()
    }

    fn min_needed_to_satisfy(&self, set: &CubeSet) -> CubeSet {
        Self {
            red: self.red.max(set.red),
            green: self.green.max(set.green),
            blue: self.blue.max(set.blue),
        }
    }
}

#[derive(PartialEq, Debug)]
struct Game {
    game_id: usize,
    cube_sets: Vec<CubeSet>,
}

impl Game {
    // Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
    fn parse(line: &str) -> Result<Self> {
        let (rem, (_, game_id, _, cube_sets)) = tuple((
            tag("Game "),
            parse_number,
            tag(": "),
            separated_list1(tag(";"), CubeSet::parse),
        ))(line)
        .map_err(|err| anyhow::format_err!("{}", err))?;
        if !rem.is_empty() {
            anyhow::bail!("line not completely parsed. remaining: {rem}");
        }
        Ok(Self { game_id, cube_sets })
    }

    fn is_possible(&self, q: &CubeSet) -> bool {
        self.cube_sets.iter().all(|set| set.is_possible(q))
    }

    fn minimum_set(&self) -> CubeSet {
        self.cube_sets.iter().fold(CubeSet::default(), |acc, set| {
            acc.min_needed_to_satisfy(set)
        })
    }
}

fn parse_number(input: &str) -> IResult<&str, usize> {
    map_res(take_while1(|c: char| c.is_digit(10)), |num_str: &str| {
        num_str.parse()
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_parse() {
        let line = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
        let game = Game::parse(line).unwrap();
        let expected = Game {
            game_id: 1,
            cube_sets: vec![
                CubeSet {
                    red: 4,
                    green: 0,
                    blue: 3,
                },
                CubeSet {
                    red: 1,
                    green: 2,
                    blue: 6,
                },
                CubeSet {
                    red: 0,
                    green: 2,
                    blue: 0,
                },
            ],
        };
        assert_eq!(game, expected)
    }

    #[test]
    fn test_game_is_possible() -> Result<()> {
        let game = Game::parse("Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue")?;
        let q1 = &CubeSet {
            red: 1,
            green: 3,
            blue: 4,
        };
        let q2 = &CubeSet {
            red: 1,
            green: 3,
            blue: 3, // saw 4 blues
        };
        assert!(game.is_possible(q1));
        assert!(!game.is_possible(q2));
        Ok(())
    }

    const INPUT: &str = r#"Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green"#;

    #[test]
    fn test_part_one_example() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 1 + 2 + 5);
        Ok(())
    }

    #[test]
    fn test_minimum_set() -> Result<()> {
        let s1 = CubeSet {
            red: 5,
            green: 0,
            blue: 1,
        };
        let s2 = CubeSet {
            red: 1,
            green: 1,
            blue: 0,
        };
        assert_eq!(
            s1.min_needed_to_satisfy(&s2),
            CubeSet {
                red: 5,
                green: 1,
                blue: 1
            }
        );
        Ok(())
    }

    #[test]
    fn test_minimum_set_default() -> Result<()> {
        let s1 = CubeSet {
            red: 5,
            green: 0,
            blue: 1,
        };
        let s2 = CubeSet::default();
        assert_eq!(s1.min_needed_to_satisfy(&s2), s1);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 48 + 12 + 1560 + 630 + 36);
        Ok(())
    }
}
