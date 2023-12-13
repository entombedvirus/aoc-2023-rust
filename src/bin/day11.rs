use anyhow::Result;
use aoc::runner;

fn main() -> Result<()> {
    runner(part_one, part_two)
}

fn part_one(input: &str) -> Result<usize> {
    process(input, 2)
}

fn part_two(input: &str) -> Result<usize> {
    process(input, 1_000_000)
}

fn process(input: &str, expansion_factor: usize) -> Result<usize> {
    let mut map = AstroMap::parse(input);
    let exp_rows: Vec<_> = map
        .rows
        .iter()
        // map row number to y axis
        .rev()
        .enumerate()
        .filter_map(|(idx, r)| r.chars().all(|ch| ch == '.').then_some(idx))
        .collect();
    let exp_cols: Vec<_> = (0..map.width())
        .filter(|col| map.rows.iter().all(|r| r.chars().nth(*col) == Some('.')))
        .collect();

    let mut total = 0;
    while let Some(src) = map.galaxies.pop() {
        let distances = map.manhattan_distance(src);
        for (idx, dist) in distances.into_iter().enumerate() {
            let dest = map.galaxies[idx];
            let num_cols = column_intersections(src, dest, &exp_cols);
            let num_rows = row_intersections(src, dest, &exp_rows);

            let num_inter = num_cols + num_rows;
            total += dist as usize + num_inter * (expansion_factor - 1);
        }
    }
    Ok(total)
}

fn column_intersections((x1, _): (usize, usize), (x2, _): (usize, usize), cols: &[usize]) -> usize {
    let xrange = if x1 > x2 { x2..x1 } else { x1..x2 };
    let mut total = 0;
    for col in cols {
        if xrange.contains(col) {
            total += 1;
        }
    }
    total
}

fn row_intersections((_, y1): (usize, usize), (_, y2): (usize, usize), rows: &[usize]) -> usize {
    let yrange = if y1 > y2 { y2..y1 } else { y1..y2 };
    let mut total = 0;
    for row in rows {
        if yrange.contains(row) {
            total += 1;
        }
    }
    total
}

#[derive(PartialEq, Eq)]
struct AstroMap {
    rows: Vec<String>,
    galaxies: Vec<(usize, usize)>, // x, y coordinate
}

impl std::fmt::Debug for AstroMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for row in &self.rows {
            writeln!(f, "{}", row)?
        }
        Ok(())
    }
}

impl AstroMap {
    fn parse(input: &str) -> Self {
        let rows: Vec<_> = input.lines().map(String::from).collect();
        Self::new(rows)
    }

    fn width(&self) -> usize {
        self.rows.first().map(|r| r.len()).unwrap_or(0)
    }

    fn new(rows: Vec<String>) -> Self {
        let num_rows = rows.len();
        let galaxies: Vec<(usize, usize)> = rows
            .iter()
            .enumerate()
            .flat_map(|(row_idx, row)| {
                row.char_indices().filter_map(move |(col_idx, ch)| {
                    if ch == '#' {
                        // row and col is flipped when turning into (x, y) coordinates
                        Some((col_idx, num_rows - row_idx - 1))
                    } else {
                        None
                    }
                })
            })
            .collect();
        Self { rows, galaxies }
    }

    fn manhattan_distance(&self, (sx, sy): (usize, usize)) -> Vec<usize> {
        self.galaxies
            .iter()
            .map(|(gx, gy)| gx.abs_diff(sx) + gy.abs_diff(sy))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#....."#;

    #[test]
    fn test_part_one() -> Result<()> {
        assert_eq!(part_one(INPUT)?, 374);
        Ok(())
    }
}
