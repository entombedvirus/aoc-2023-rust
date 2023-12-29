#![allow(unused, dead_code)]

use std::collections::{BTreeMap, VecDeque};

use anyhow::{Context, Result};
use aoc::{must_parse, runner};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, newline, one_of},
    combinator::{map, recognize},
    multi::separated_list1,
    sequence::{separated_pair, tuple},
};

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    let mut p = Puzzle::parse(input)?;
    let mut counter = SignalCounter::new();
    for _ in 0..1000 {
        p.press_button(&mut counter);
    }

    Ok(counter.low_pulses * counter.high_pulses)
}

fn part_two(_input: &str) -> Result<u32> {
    todo!()
}

#[derive(Debug)]
struct Puzzle<'i> {
    modules: BTreeMap<&'i str, Module<'i>>,
    forward_edges: BTreeMap<&'i str, Vec<&'i str>>,
    reverse_edges: BTreeMap<&'i str, Vec<&'i str>>,
}

impl<'i> Puzzle<'i> {
    fn parse(input: &'i str) -> Result<Self> {
        let parse_module_type_and_name =
            tuple((alt((recognize(one_of("&%")), tag("broadcast"))), alpha1));
        let parse_destinations = separated_list1(tag(", "), alpha1);

        let parse_module = map(
            separated_pair(parse_module_type_and_name, tag(" -> "), parse_destinations),
            |((mtype, name), destinations)| {
                let module = match mtype {
                    "%" => Module::FlipFlop { name, is_on: false },
                    "&" => Module::Conjunction {
                        name,
                        // these will be filled out when when constructing the puzzle
                        inputs: Default::default(),
                    },
                    "broadcast" => Module::Broadcast,
                    _ => unreachable!(),
                };
                (module, destinations)
            },
        );
        let parser = map(separated_list1(newline, parse_module), Puzzle::new);
        must_parse(parser, input)
    }

    fn new(module_pairs: Vec<(Module<'i>, Vec<&'i str>)>) -> Self {
        let mut forward_edges = BTreeMap::new();
        let mut reverse_edges = BTreeMap::new();
        let mut modules = BTreeMap::new();

        for (module, destinations) in module_pairs {
            let src_module = module.name();
            for dest in destinations {
                forward_edges.entry(src_module).or_insert(vec![]).push(dest);
                reverse_edges.entry(dest).or_insert(vec![]).push(src_module);
            }
            modules.insert(src_module, module);
        }

        // default inputs of Conjunction modules to low pulse
        for module in modules.values_mut() {
            if let Module::Conjunction {
                ref name,
                ref mut inputs,
            } = module
            {
                if let Some(redges) = reverse_edges.get(name) {
                    inputs.extend(redges.iter().map(|redge| (*redge, Pulse::Low)));
                }
            }
        }

        Self {
            modules,
            forward_edges,
            reverse_edges,
        }
    }

    fn press_button(&mut self, counter: &mut SignalCounter) {
        let mut queue = VecDeque::new();
        queue.push_back(Signal {
            from: "button",
            to: "broadcaster",
            pulse: Pulse::Low,
        });

        while let Some(signal) = queue.pop_front() {
            counter.incr(signal.pulse);
            self.propagate(signal, &mut queue);
        }
    }

    fn propagate(&mut self, signal: Signal<'i>, queue: &mut VecDeque<Signal<'i>>) {
        let Some(module) = self.modules.get_mut(signal.to) else {
            // sink module that has no state and is not connected to anything
            return;
        };

        let new_pulse = match module {
            Module::FlipFlop { name, is_on } => {
                if signal.pulse == Pulse::Low {
                    // toggle boolean
                    *is_on ^= true;
                    Some(if *is_on { Pulse::High } else { Pulse::Low })
                } else {
                    None
                }
            }
            Module::Conjunction { name, inputs } => {
                inputs.entry(signal.from).and_modify(|s| *s = signal.pulse);
                Some(
                    if inputs.values().all(|remembered| remembered == &Pulse::High) {
                        Pulse::Low
                    } else {
                        Pulse::High
                    },
                )
            }
            Module::Broadcast => Some(signal.pulse),
        };

        if let Some(new_pulse) = new_pulse {
            let connected_to = &self.forward_edges[signal.to];
            let new_signals = connected_to.iter().map(|dest| Signal {
                from: signal.to,
                to: dest,
                pulse: new_pulse,
            });
            queue.extend(new_signals);
        }
    }
}

#[derive(Debug)]
struct SignalCounter {
    low_pulses: usize,
    high_pulses: usize,
}

impl SignalCounter {
    fn new() -> Self {
        SignalCounter {
            low_pulses: 0,
            high_pulses: 0,
        }
    }

    fn incr(&mut self, pulse: Pulse) {
        match pulse {
            Pulse::Low => self.low_pulses += 1,
            Pulse::High => self.high_pulses += 1,
        }
    }
}

#[derive(Debug)]
struct Signal<'i> {
    from: &'i str,
    to: &'i str,
    pulse: Pulse,
}

#[derive(Debug)]
enum Module<'i> {
    FlipFlop {
        name: &'i str,
        is_on: bool,
    },
    Conjunction {
        name: &'i str,
        inputs: BTreeMap<&'i str, Pulse>,
    },
    Broadcast,
}

impl<'i> Module<'i> {
    fn name(&self) -> &'i str {
        match self {
            Self::FlipFlop { name, .. } => name,
            Self::Conjunction { name, .. } => name,
            Self::Broadcast { .. } => "broadcaster",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pulse {
    Low,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a"#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 32000000);
        Ok(())
    }
}
