#![allow(unused, dead_code)]

use std::{
    collections::{BTreeMap, VecDeque},
    hash::{BuildHasher, Hasher, RandomState},
};

use anyhow::{Context, Result};
use aoc::{must_parse, runner, wait};
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

fn part_two(input: &str) -> Result<u64> {
    let mut p = Puzzle::parse(input)?;

    // rx is hooked up to a single module
    let rx_parents = &p.reverse_edges["rx"];
    assert!(rx_parents.len() == 1);

    // that parent module is a conjunction module
    let zh = &p.modules[rx_parents[0]];
    assert!(zh.is_conjunction_module());

    // which means that all inputs to zh must emit a low pulse
    // in order for it to emit a low pulse to rx. Let's monitor
    // how often each of input emit a high pulse individually.
    let zh_inputs = &p.reverse_edges[zh.name()];
    let mut counter = SignalCounter::new();
    for input in zh_inputs {
        counter.monitor(Signal {
            from: input,
            to: zh.name(),
            pulse: Pulse::High,
        });
    }

    // press button enough times that each of zh's input is
    // Pulse:High at least once
    while counter.monitored_counts.values().any(|v| *v == 0) {
        p.press_button(&mut counter);
    }

    counter
        .monitored_counts
        .values()
        .copied()
        .reduce(compute_lcm)
        .context("Unable to find lowest common denominator")
}

// See: https://en.wikipedia.org/wiki/Least_common_multiple
fn compute_lcm(a: u64, b: u64) -> u64 {
    a * (b / compute_gcd_euclid(a, b))
}

// See: https://en.wikipedia.org/wiki/Euclidean_algorithm
fn compute_gcd_euclid(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp
    }
    a
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

    fn press_button(&mut self, counter: &mut SignalCounter<'i>) {
        let mut queue = VecDeque::new();
        queue.push_back(Signal {
            from: "button",
            to: "broadcaster",
            pulse: Pulse::Low,
        });

        counter.button_press_count += 1;
        while let Some(signal) = queue.pop_front() {
            counter.incr(signal);
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

    fn state_hash(&self) -> u64 {
        let rand_state = RandomState::new();
        let mut hasher = rand_state.build_hasher();

        for module in self.modules.values() {
            match module {
                Module::FlipFlop { is_on, .. } => hasher.write_u8(*is_on as u8),
                Module::Conjunction { inputs, .. } => {
                    for input_state in inputs.values() {
                        hasher.write_u8(*input_state as u8);
                    }
                }
                Module::Broadcast => (),
            }
        }
        hasher.finish()
    }
}

#[derive(Debug)]
struct SignalCounter<'i> {
    low_pulses: usize,
    high_pulses: usize,
    button_press_count: u64,
    monitored_counts: BTreeMap<Signal<'i>, u64>,
}

impl<'i> SignalCounter<'i> {
    fn new() -> Self {
        SignalCounter {
            low_pulses: 0,
            high_pulses: 0,
            button_press_count: 0,
            monitored_counts: Default::default(),
        }
    }

    fn incr(&mut self, sig: Signal<'i>) {
        match sig.pulse {
            Pulse::Low => self.low_pulses += 1,
            Pulse::High => self.high_pulses += 1,
        }
        self.monitored_counts.entry(sig).and_modify(|v| {
            if *v == 0 {
                *v = self.button_press_count;
            }
        });
    }

    fn monitor(&mut self, sig: Signal<'i>) {
        self.monitored_counts.entry(sig).or_insert(0);
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
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

    fn is_conjunction_module(&self) -> bool {
        match self {
            Self::Conjunction { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
