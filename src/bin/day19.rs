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

fn part_two(input: &str) -> Result<u64> {
    let p = Puzzle::parse(input)?;
    let valid_range = 1..4001;

    let mut count = 0;
    let mut queue = vec![("in", CountState::new(&valid_range))];

    // perform DFS of the graph summing up possibilities every time we reach the "A" terminal node
    while let Some((name, state)) = queue.pop() {
        let w = &p.workflows[name];
        for (child, state) in w.child_with_state(&state) {
            match child {
                "A" => count += state.num_possibilities(),
                "R" => continue,
                other => queue.push((other, state)),
            }
        }
    }

    Ok(count)
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
        let parse_op = map(tuple((one_of("<>"), complete::u16)), |(ch, num)| match ch {
            '<' => Operation::LessThan(num),
            '>' => Operation::GreaterThan(num),
            unknown => unreachable!("unknown operation: {}", unknown),
        });
        let parse_cond = map(
            tuple((parse_part_field, parse_op, complete::char(':'))),
            |(field, op, _)| Condition { field, op },
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
                complete::u16,
                complete::char(','),
                tag("m="),
                complete::u16,
                complete::char(','),
                tag("a="),
                complete::u16,
                complete::char(','),
                tag("s="),
                complete::u16,
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

    fn num_possibilities(&self, valid_range: std::ops::Range<u16>) -> u64 {
        let mut count = 0;

        let mut queue = vec![("in", CountState::new(&valid_range))];
        while let Some((name, state)) = queue.pop() {
            let w = &self.workflows[name];
            for (child, state) in w.child_with_state(&state) {
                match child {
                    "A" => count += state.num_possibilities(),
                    "R" => continue,
                    other => queue.push((other, state)),
                }
            }
        }

        count
    }
}

#[derive(Debug, Clone)]
struct CountState {
    x: SplittableRange,
    m: SplittableRange,
    a: SplittableRange,
    s: SplittableRange,
}

impl CountState {
    fn new(valid_range: &std::ops::Range<u16>) -> Self {
        Self {
            x: SplittableRange::new(valid_range),
            m: SplittableRange::new(valid_range),
            a: SplittableRange::new(valid_range),
            s: SplittableRange::new(valid_range),
        }
    }

    fn num_possibilities(&self) -> u64 {
        let Self { x, m, a, s } = self;
        [x, m, a, s]
            .into_iter()
            .map(|srange| srange.len() as u64)
            .product()
    }

    /// restrict_and_shrink retruns the subset of `self` that matches `cond`. It also shrinks `self` to
    /// the parts that are not covered by `cond`.
    fn restrict_and_shrink(&mut self, cond: &Condition) -> CountState {
        let mut restricted = self.clone();
        let (self_range, restricted_range) = match cond.field {
            PartField::X => (&mut self.x, &mut restricted.x),
            PartField::M => (&mut self.m, &mut restricted.m),
            PartField::A => (&mut self.a, &mut restricted.a),
            PartField::S => (&mut self.s, &mut restricted.s),
        };
        restricted_range.apply(&cond.op);
        self_range.apply(&cond.op.negate());

        restricted
    }
}

#[derive(Debug, Clone)]
struct SplittableRange {
    points: BTreeMap<u16, u16>,
}

impl SplittableRange {
    fn new(valid_range: &std::ops::Range<u16>) -> SplittableRange {
        let mut points = BTreeMap::new();
        points.insert(valid_range.start, valid_range.end);
        Self { points }
    }

    fn len(&self) -> u16 {
        self.points.iter().map(|(start, end)| *end - *start).sum()
    }

    fn apply(&mut self, op: &Operation) {
        match *op {
            Operation::LessThan(x) => {
                self.points.retain(|&start, _| start < x);
                if let Some(entry) = self.points.last_entry() {
                    if (*entry.get()) > x {
                        let closed_start = *entry.key();
                        entry.remove();
                        self.points.insert(closed_start, x);
                    }
                }
            }
            Operation::GreaterThan(x) => {
                self.points.retain(|_, end| *end > x);
                if let Some(entry) = self.points.first_entry() {
                    if (*entry.key()) <= x {
                        let open_end = *entry.get();
                        entry.remove();
                        self.points.insert(x + 1, open_end);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct Part {
    x: u16,
    m: u16,
    a: u16,
    s: u16,
}
impl Part {
    fn part_sum(&self) -> u32 {
        let &Self { x, m, a, s } = self;
        [x, m, a, s].into_iter().map(|x| x as u32).sum()
    }
}

#[derive(Debug)]
struct Workflow<'i> {
    name: &'i str,
    steps: Vec<Step<'i>>,
}
impl<'i> Workflow<'i> {
    fn run(&self, part: &Part) -> &str {
        self.steps
            .iter()
            .find_map(|step| step.run(part))
            .expect("last step is always expected to match")
    }

    fn child_with_state(
        &'i self,
        input: &CountState,
    ) -> impl Iterator<Item = (&'i str, CountState)> + 'i {
        self.steps
            .iter()
            .scan(input.clone(), |state, st| match &st.cond {
                Some(cond) => {
                    let child_state = state.restrict_and_shrink(cond);
                    Some((st.dest_workflow, child_state))
                }
                None => Some((st.dest_workflow, state.clone())),
            })
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
}

impl Condition {
    fn eval(&self, part: &Part) -> bool {
        let op = match self.field {
            PartField::X => part.x,
            PartField::M => part.m,
            PartField::A => part.a,
            PartField::S => part.s,
        };
        self.op.eval(op)
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
    LessThan(u16),
    GreaterThan(u16),
}

impl Operation {
    fn eval(&self, op1: u16) -> bool {
        match self {
            Operation::LessThan(op2) => op1 < *op2,
            Operation::GreaterThan(op2) => op1 > *op2,
        }
    }

    fn negate(&self) -> Self {
        match self {
            Operation::LessThan(x) => Operation::GreaterThan(*x - 1),
            Operation::GreaterThan(x) => Operation::LessThan(*x + 1),
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

    #[test]
    fn test_part_two() -> Result<()> {
        assert_eq!(part_two(INPUT)?, 167409079868000);
        Ok(())
    }
}
