use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Start,
    Splitter,
    Beam,
}

impl Cell {
    fn from_char(c: char) -> Result<Self> {
        match c {
            'S' => Ok(Cell::Start),
            '.' => Ok(Cell::Empty),
            '^' => Ok(Cell::Splitter),
            '|' => Ok(Cell::Beam),
            _ => Err(anyhow!("Invalid cell character: {}", c)),
        }
    }

    fn to_char(self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::Start => 'S',
            Cell::Splitter => '^',
            Cell::Beam => '|',
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

fn parse_input(file_path: &str) -> Result<Vec<Vec<Cell>>> {
    let contents = std::fs::read_to_string(file_path)?;
    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .map(Cell::from_char)
                .collect()
        })
        .collect()
}

// Fast DP solution: track beams with their multiplicity (how many timelines they represent)
fn count_timelines_dp(grid: &mut [Vec<Cell>]) -> Result<(usize, u64)> {
    if grid.is_empty() {
        return Ok((0, 0));
    }

    let mut split_count = 0;

    // Find the Start (S) in the first line
    let first_line = &grid[0];
    let start_idx = match first_line.iter().position(|&cell| cell == Cell::Start) {
        Some(idx) => idx,
        None => return Ok((0, 0)),
    };

    // Track active beams: (row, col, multiplicity)
    // multiplicity = how many timelines this beam represents
    let mut active_beams: Vec<(usize, usize, u64)> = vec![];
    
    // Initialize with the first beam position (represents 1 timeline)
    if grid.len() > 1 {
        grid[1][start_idx] = Cell::Beam;
        active_beams.push((1, start_idx, 1));
    }

    // Process each line from the second line onwards
    for line_idx in 1..grid.len() - 1 {
        let next_line_idx = line_idx + 1;
        let next_line = &mut grid[next_line_idx];

        // Use a HashMap to merge beams at the same position
        let mut beam_map: HashMap<usize, u64> = HashMap::new();
        
        // Track which positions have had splitters for counting purposes only
        let mut split_positions = HashSet::new();

        // Process active beams - each beam carries its multiplicity
        for (beam_row, beam_col, multiplicity) in &active_beams {
            if *beam_row == line_idx {
                // Check if the next line at this position is a splitter
                if next_line[*beam_col] == Cell::Splitter {
                    // Count this split only once per position
                    if split_positions.insert(*beam_col) {
                        split_count += 1;
                    }
                    
                    // Place beams at both +1 and -1 positions
                    // Each new beam inherits the same multiplicity (same number of timelines)
                    if *beam_col > 0 {
                        next_line[*beam_col - 1] = Cell::Beam;
                        *beam_map.entry(*beam_col - 1).or_insert(0) += *multiplicity;
                    }
                    if *beam_col < next_line.len() - 1 {
                        next_line[*beam_col + 1] = Cell::Beam;
                        *beam_map.entry(*beam_col + 1).or_insert(0) += *multiplicity;
                    }
                } else {
                    // Place beam at the same index in the next line
                    next_line[*beam_col] = Cell::Beam;
                    // Beam continues with same multiplicity, merge if multiple beams reach same position
                    *beam_map.entry(*beam_col).or_insert(0) += *multiplicity;
                }
            }
        }
        
        // Convert beam_map back to active_beams
        active_beams = beam_map.into_iter()
            .map(|(col, mult)| (next_line_idx, col, mult))
            .collect();
    }

    // Sum up the multiplicities of all final beams
    let total_timelines: u64 = active_beams.iter().map(|(_, _, m)| m).sum();
    
    Ok((split_count, total_timelines))
}

pub fn run() -> Result<()> {
    // Test with small example first
    println!("Testing with small example:");
    let mut test_grid = parse_input("assets/day07test.txt")?;
    let (test_splits, test_timelines) = count_timelines_dp(&mut test_grid)?;
    println!("  Split count: {} (expected: 21)", test_splits);
    println!("  Unique timelines: {} (expected: 40)", test_timelines);
    println!();
    
    // Run with full input
    println!("Running with full input:");
    let mut grid = parse_input("assets/day07splitter.txt")?;
    
    let start = std::time::Instant::now();
    let (split_count, timeline_count) = count_timelines_dp(&mut grid)?;
    let elapsed = start.elapsed();
    
    println!("  Split count: {}", split_count);
    println!("  Unique timelines: {}", timeline_count);
    println!("  Time elapsed: {:?}", elapsed);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_example() {
        let mut test_grid = parse_input("assets/day07test.txt")
            .expect("Failed to read test input file");
        
        let (split_count, timeline_count) = count_timelines_dp(&mut test_grid)
            .expect("Failed to count timelines");
        
        assert_eq!(split_count, 21, "Test split count should be 21");
        assert_eq!(timeline_count, 40, "Test timeline count should be 40");
    }

    #[test]
    fn test_full_solution() {
        let mut grid = parse_input("assets/day07splitter.txt")
            .expect("Failed to read input file");
        
        let (split_count, timeline_count) = count_timelines_dp(&mut grid)
            .expect("Failed to count timelines");
        
        assert_eq!(split_count, 1651, "Full split count should be 1651");
        assert_eq!(timeline_count, 108924003331749, "Full timeline count should be 108924003331749");
    }
}
