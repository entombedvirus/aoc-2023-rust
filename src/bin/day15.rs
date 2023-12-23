use anyhow::Result;
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    Ok(input
        .trim_end()
        .split(',')
        .map(Label)
        .map(|l| l.hash() as u32)
        .sum())
}

fn part_two(input: &str) -> Result<usize> {
    use Instruction::*;
    let instructions = input.trim_end().split(',').map(Instruction::parse);

    let mut boxes = vec![Vec::<(Label<'_>, FocalLength)>::new(); 256];
    for i in instructions {
        let lenses = &mut boxes[i.hash() as usize];
        match i {
            Equals(l, f) => match lenses.iter_mut().find(|(li, _)| li == &l) {
                Some(x) => {
                    *x = (l, f);
                }
                None => {
                    lenses.push((l, f));
                }
            },
            Dash(ref l) => {
                if let Some(x) = lenses.iter_mut().position(|(li, _)| li == l) {
                    lenses.remove(x);
                }
            }
        };
    }

    Ok(boxes
        .into_iter()
        .enumerate()
        .flat_map(move |(box_idx, lenses)| {
            lenses
                .into_iter()
                .enumerate()
                .map(move |(lens_idx, (_, f))| (1 + box_idx) * (lens_idx + 1) * f as usize)
        })
        .sum())
}

type FocalLength = u8;

#[derive(Debug, Clone)]
enum Instruction<'i> {
    Equals(Label<'i>, FocalLength),
    Dash(Label<'i>),
}

impl<'i> Instruction<'i> {
    fn parse(as_str: &'i str) -> Self {
        use Instruction::*;
        let op_idx = as_str.find(['-', '=']).expect("operation not found");
        let label = Label(&as_str[0..op_idx]);
        match as_str.as_bytes()[op_idx] {
            b'=' => Equals(label, as_str.as_bytes()[op_idx + 1] - b'0'),
            b'-' => Dash(label),
            unknown => unreachable!("only searched for either dash or minus: {unknown:?}"),
        }
    }

    fn hash(&self) -> u8 {
        self.label().hash()
    }

    fn label(&self) -> &Label<'i> {
        use Instruction::*;
        match self {
            Equals(l, _) => l,
            Dash(l) => l,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Label<'i>(&'i str);

impl<'i> Label<'i> {
    fn hash(&self) -> u8 {
        self.0
            .as_bytes()
            .iter()
            .copied()
            .fold(0u32, |acc, ch| ((acc + ch as u32) * 17) % 256) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 1320);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 145);
        Ok(())
    }
}
