#![allow(unused, dead_code)]

use std::collections::BTreeMap;

use anyhow::Result;
use aoc::{must_parse, runner};
use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{self, alpha1, newline, one_of},
    combinator::{map, opt},
    multi::{count, separated_list1},
    sequence::{delimited, separated_pair, tuple},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<u32> {
    let p = Puzzle::parse(input)?;
    let mut bins = BTreeMap::new();
    p.run(&mut bins);
    Ok(bins["A"].iter().map(|p| p.part_sum()).sum())
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Puzzle<'i> {
    workflows: BTreeMap<&'i str, Workflow<'i>>,
    parts: Vec<Part>,
}

impl<'i> Puzzle<'i> {
    fn parse(input: &'i str) -> Result<Self> {
        let parse_part_field = map(one_of("xmas"), |ch| match ch {
            'x' => PartField::X,
            'm' => PartField::M,
            'a' => PartField::A,
            's' => PartField::S,
            unknown => unreachable!("unknown part field: {}", unknown),
        });
        let parse_op = map(one_of("<>"), |ch| match ch {
            '<' => Operation::LessThan,
            '>' => Operation::GreaterThan,
            unknown => unreachable!("unknown operation: {}", unknown),
        });
        let parse_cond = map(
            tuple((
                parse_part_field,
                parse_op,
                complete::u32,
                complete::char(':'),
            )),
            |(field, op, operand, _)| Condition { field, op, operand },
        );
        let parse_step = map(tuple((opt(parse_cond), alpha1)), |(cond, dest_workflow)| {
            Step {
                cond,
                dest_workflow,
            }
        });
        let parse_workflow = map(
            tuple((
                alpha1,
                delimited(
                    complete::char('{'),
                    separated_list1(complete::char(','), parse_step),
                    complete::char('}'),
                ),
            )),
            |(name, steps)| Workflow { name, steps },
        );
        let parse_workflows = map(
            separated_list1(newline, parse_workflow),
            |ws: Vec<Workflow>| {
                ws.into_iter()
                    .map(|w| (w.name, w))
                    .collect::<BTreeMap<&str, Workflow>>()
            },
        );
        let parse_part = map(
            tuple((
                tag("x="),
                complete::u32,
                complete::char(','),
                tag("m="),
                complete::u32,
                complete::char(','),
                tag("a="),
                complete::u32,
                complete::char(','),
                tag("s="),
                complete::u32,
            )),
            |(_, x, _, _, m, _, _, a, _, _, s)| Part { x, m, a, s },
        );
        let parse_parts = separated_list1(
            newline,
            delimited(complete::char('{'), parse_part, complete::char('}')),
        );
        let parser = map(
            separated_pair(parse_workflows, count(newline, 2), parse_parts),
            |(workflows, parts): (BTreeMap<&str, Workflow>, Vec<Part>)| Self { workflows, parts },
        );
        must_parse(parser, input)
    }

    fn run(&'i self, bins: &mut BTreeMap<&'i str, Vec<&'i Part>>) {
        for part in &self.parts {
            let mut workflow_name = "in";
            while workflow_name != "A" && workflow_name != "R" {
                workflow_name = self.workflows[workflow_name].run(part);
            }
            bins.entry(workflow_name)
                .and_modify(|mut parts| parts.push(part))
                .or_insert(vec![part]);
        }
    }
}

#[derive(Debug)]
struct Part {
    x: u32,
    m: u32,
    a: u32,
    s: u32,
}
impl Part {
    fn part_sum(&self) -> u32 {
        let &Self { x, m, a, s } = self;
        x + m + a + s
    }
}

#[derive(Debug)]
struct Workflow<'i> {
    name: &'i str,
    steps: Vec<Step<'i>>,
}
impl Workflow<'_> {
    fn run(&self, part: &Part) -> &str {
        self.steps
            .iter()
            .find_map(|step| step.run(part))
            .expect("last step is always expected to match")
    }
}

#[derive(Debug)]
struct Step<'i> {
    cond: Option<Condition>,
    dest_workflow: &'i str,
}
impl Step<'_> {
    fn run(&self, part: &Part) -> Option<&str> {
        if let Some(cond) = &self.cond {
            cond.eval(part).then_some(self.dest_workflow)
        } else {
            Some(self.dest_workflow)
        }
    }
}

#[derive(Debug)]
struct Condition {
    field: PartField,
    op: Operation,
    operand: u32,
}
impl Condition {
    fn eval(&self, part: &Part) -> bool {
        let op1 = match self.field {
            PartField::X => part.x,
            PartField::M => part.m,
            PartField::A => part.a,
            PartField::S => part.s,
        };
        self.op.eval(op1, self.operand)
    }
}

#[derive(Debug)]
enum PartField {
    X,
    M,
    A,
    S,
}

#[derive(Debug)]
enum Operation {
    LessThan,
    GreaterThan,
}
impl Operation {
    fn eval(&self, op1: u32, op2: u32) -> bool {
        match self {
            Operation::LessThan => op1 < op2,
            Operation::GreaterThan => op1 > op2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 19114);
        Ok(())
    }
}
