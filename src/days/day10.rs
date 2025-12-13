use anyhow::{anyhow, Context, Result};
use std::fmt;
use std::fs;

#[derive(Clone)]
pub struct Machine {
    pub goal_lights: Vec<bool>,        // Goal state of lights
    pub current_lights: Vec<bool>,          // Current state of lights (initially all false)
    pub goal_joltage: Vec<usize>,    // Goal state of joltage (from curly braces)
    pub current_joltage: Vec<usize>, // Current state of joltage (initially all 0)
    pub buttons: Vec<Vec<usize>>,
}

impl Machine {
    // Methods removed - using Gaussian elimination instead
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "Joltage:")?;
        write!(f, "- current: {{")?;
        for (i, &jolt) in self.current_joltage.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", jolt)?;
        }
        writeln!(f, "}}")?;
        write!(f, "- goal: {{")?;
        for (i, &jolt) in self.goal_joltage.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", jolt)?;
        }
        writeln!(f, "}}")?;
        
        writeln!(f, "Lights:")?;
        write!(f, "- current: [")?;
        for &light in &self.current_lights {
            write!(f, "{}", if light { '#' } else { '.' })?;
        }
        writeln!(f, "]")?;
        write!(f, "- goal: [")?;
        for &light in &self.goal_lights {
            write!(f, "{}", if light { '#' } else { '.' })?;
        }
        writeln!(f, "]")?;
        
        writeln!(f, "Buttons:")?;
        write!(f, "- ")?;
        for (i, button) in self.buttons.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "(")?;
            for (j, &idx) in button.iter().enumerate() {
                if j > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", idx)?;
            }
            write!(f, ")")?;
        }
        writeln!(f, "")?;
        
        Ok(())
    }
}

fn parse_input(filename: &str) -> Result<Vec<Machine>> {
    let content = fs::read_to_string(filename)
        .context(format!("Failed to read file: {}", filename))?;

    let machines: Vec<Machine> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            let line = line.trim();
            
            // Extract lights part: [.##.]
            let lights_start = line.find('[')
                .ok_or_else(|| anyhow!("Line {}: missing '[' for lights", i + 1))?;
            let lights_end = line.find(']')
                .ok_or_else(|| anyhow!("Line {}: missing ']' for lights", i + 1))?;
            
            let lights_str = &line[lights_start + 1..lights_end];
            let solution: Vec<bool> = lights_str
                .chars()
                .map(|c| match c {
                    '.' => Ok(false),
                    '#' => Ok(true),
                    _ => Err(anyhow!("Line {}: invalid light character '{}'", i + 1, c)),
                })
                .collect::<Result<Vec<_>>>()?;
            
            // Initialize current state to all off (false)
            let current = vec![false; solution.len()];
            
            // Extract buttons: (3) (1,3) (2) etc.
            let mut buttons = Vec::new();
            let mut pos = lights_end + 1;
            
            while pos < line.len() {
                // Skip whitespace
                while pos < line.len() && line.chars().nth(pos).unwrap().is_whitespace() {
                    pos += 1;
                }
                
                // Check if we've reached the curly braces part
                if pos < line.len() && line.chars().nth(pos).unwrap() == '{' {
                    break;
                }
                
                // Find next opening parenthesis
                if let Some(open_paren) = line[pos..].find('(') {
                    let button_start = pos + open_paren;
                    let button_end = line[button_start..]
                        .find(')')
                        .ok_or_else(|| anyhow!("Line {}: missing ')' for button", i + 1))?;
                    
                    let button_str = &line[button_start + 1..button_start + button_end];
                    let button_indices: Vec<usize> = if button_str.is_empty() {
                        Vec::new()
                    } else {
                        button_str
                            .split(',')
                            .map(|s| {
                                s.trim()
                                    .parse::<usize>()
                                    .context(format!("Line {}: invalid button index '{}'", i + 1, s))
                            })
                            .collect::<Result<Vec<_>>>()?
                    };
                    
                    buttons.push(button_indices);
                    pos = button_start + button_end + 1;
                } else {
                    break;
                }
            }
            
            // Extract joltage goal from curly braces: {3,5,4,7}
            let goal_joltage = if let Some(brace_start) = line.find('{') {
                let brace_end = line[brace_start..]
                    .find('}')
                    .ok_or_else(|| anyhow!("Line {}: missing '}}' for joltage", i + 1))?;
                
                let joltage_str = &line[brace_start + 1..brace_start + brace_end];
                if joltage_str.is_empty() {
                    Vec::new()
                } else {
                    joltage_str
                        .split(',')
                        .map(|s| {
                            s.trim()
                                .parse::<usize>()
                                .context(format!("Line {}: invalid joltage value '{}'", i + 1, s))
                        })
                        .collect::<Result<Vec<_>>>()?
                }
            } else {
                Vec::new()
            };
            
            // Initialize current joltage to all 0
            let current_joltage = vec![0; goal_joltage.len()];
            
            Ok(Machine { 
                goal_lights: solution, 
                current_lights: current, 
                goal_joltage,
                current_joltage,
                buttons 
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(machines)
}

// Old brute-force methods removed - using Gaussian elimination now

/// Solve a machine's joltage using Gaussian elimination with free variable optimization
/// Returns the minimum number of button presses needed
fn solve_joltage(machine: &Machine) -> usize {
    if machine.goal_joltage.is_empty() {
        return 0;
    }
    
    let num_counters = machine.goal_joltage.len();
    let num_buttons = machine.buttons.len();
    
    // Build the augmented matrix [A | b]
    let mut matrix: Vec<Vec<f64>> = vec![vec![0.0; num_buttons + 1]; num_counters];
    
    // Fill the matrix
    for (counter_idx, row) in matrix.iter_mut().enumerate() {
        for (button_idx, button) in machine.buttons.iter().enumerate() {
            if button.contains(&counter_idx) {
                row[button_idx] = 1.0;
            }
        }
        row[num_buttons] = machine.goal_joltage[counter_idx] as f64;
    }
    
    // Track which columns have pivots (basic variables)
    let mut pivot_cols = vec![];
    let mut pivot_rows = vec![];
    
    // Forward elimination to reduced row echelon form (RREF)
    let mut current_row = 0;
    for col in 0..num_buttons {
        // Find pivot in this column at or below current_row
        let pivot_row = (current_row..num_counters)
            .find(|&row| matrix[row][col].abs() > 1e-10);
        
        if let Some(pivot_row) = pivot_row {
            // Swap rows if needed
            if pivot_row != current_row {
                matrix.swap(current_row, pivot_row);
            }
            
            pivot_cols.push(col);
            pivot_rows.push(current_row);
            
            // Normalize pivot row
            let pivot_val = matrix[current_row][col];
            for j in 0..=num_buttons {
                matrix[current_row][j] /= pivot_val;
            }
            
            // Eliminate below and above the pivot
            for row in 0..num_counters {
                if row != current_row && matrix[row][col].abs() > 1e-10 {
                    let factor = matrix[row][col];
                    for j in 0..=num_buttons {
                        matrix[row][j] -= factor * matrix[current_row][j];
                    }
                }
            }
            
            current_row += 1;
            if current_row >= num_counters {
                break;
            }
        }
    }
    
    // Identify free variables (columns without pivots)
    let mut is_free = vec![true; num_buttons];
    for &col in &pivot_cols {
        is_free[col] = false;
    }
    
    let free_vars: Vec<usize> = (0..num_buttons).filter(|&i| is_free[i]).collect();
    
    // Debug: print matrix and free variables
    #[cfg(debug_assertions)]
    if false {
        println!("  RREF Matrix:");
        for row in matrix.iter() {
            print!("    ");
            for val in row {
                print!("{:6.2} ", val);
            }
            println!();
        }
        println!("  Pivot cols: {:?}", pivot_cols);
        println!("  Free vars: {:?}", free_vars);
    }
    
    // If no free variables, just read off the solution
    if free_vars.is_empty() {
        let mut solution = vec![0.0; num_buttons];
        for (&pivot_col, &pivot_row) in pivot_cols.iter().zip(pivot_rows.iter()) {
            solution[pivot_col] = matrix[pivot_row][num_buttons];
        }
        
        let total: usize = solution.iter()
            .map(|&x| x.round().max(0.0) as usize)
            .sum();
        return total;
    }
    
    // Search over small values of free variables to find minimum
    // Use a reasonable search limit based on the maximum goal value
    let max_goal = *machine.goal_joltage.iter().max().unwrap_or(&0);
    let goal_sum: usize = machine.goal_joltage.iter().sum();
    
    // Heuristic: search up to the max of (max_goal, goal_sum / num_buttons)
    // but cap it at a reasonable value to avoid infinite loops
    let search_limit = max_goal.max(goal_sum / num_buttons.max(1)).min(200);
    
    let mut best_sum = usize::MAX;
    
    // Helper function to try a specific assignment of free variables
    let try_free_assignment = |free_values: &[usize]| -> Option<usize> {
        let mut solution = vec![0.0; num_buttons];
        
        // Set free variables
        for (i, &free_var) in free_vars.iter().enumerate() {
            solution[free_var] = free_values[i] as f64;
        }
        
        // Compute basic variables from RREF
        for (&pivot_col, &pivot_row) in pivot_cols.iter().zip(pivot_rows.iter()) {
            let mut val = matrix[pivot_row][num_buttons];
            for col in 0..num_buttons {
                if col != pivot_col {
                    val -= matrix[pivot_row][col] * solution[col];
                }
            }
            solution[pivot_col] = val;
        }
        
        // Check if all values are non-negative
        if solution.iter().any(|&x| x < -0.01) {
            return None;
        }
        
        // Round to integers
        let int_solution: Vec<usize> = solution.iter()
            .map(|&x| x.round().max(0.0) as usize)
            .collect();
        
        // Verify solution
        let mut computed = vec![0usize; num_counters];
        for (button_idx, &presses) in int_solution.iter().enumerate() {
            for _ in 0..presses {
                for &counter_idx in &machine.buttons[button_idx] {
                    if counter_idx < num_counters {
                        computed[counter_idx] += 1;
                    }
                }
            }
        }
        
        if computed == machine.goal_joltage {
            Some(int_solution.iter().sum())
        } else {
            None
        }
    };
    
    // Try all combinations of free variable values with pruning
    fn enumerate_combinations(
        free_vars_count: usize,
        search_limit: usize,
        current: &mut Vec<usize>,
        try_fn: &impl Fn(&[usize]) -> Option<usize>,
        best: &mut usize,
    ) {
        if current.len() == free_vars_count {
            if let Some(sum) = try_fn(current) {
                *best = (*best).min(sum);
            }
            return;
        }
        
        // Calculate current partial sum
        let current_sum: usize = current.iter().sum();
        
        for val in 0..=search_limit {
            // Prune if current partial sum already exceeds best
            if current_sum + val >= *best {
                break;
            }
            
            current.push(val);
            enumerate_combinations(free_vars_count, search_limit, current, try_fn, best);
            current.pop();
        }
    }
    
    let mut current = Vec::new();
    enumerate_combinations(free_vars.len(), search_limit, &mut current, &try_free_assignment, &mut best_sum);
    
    // If no solution found, return 0 (should not happen with correct input)
    if best_sum == usize::MAX {
        eprintln!("WARNING: No solution found for machine!");
        return 0;
    }
    
    best_sum
}

/// Day 10: Exercise description
pub fn run() -> Result<()> {
    // Part 1
    println!("=== Part 1 ===");
    let machines1 = parse_input("assets/day10machines1.txt")?;
    println!("Parsed {} machines", machines1.len());
    
    let mut total1 = 0;
    for (i, machine) in machines1.into_iter().enumerate() {
        let presses = solve_joltage(&machine);
        println!("Machine {}: {} presses", i + 1, presses);
        total1 += presses;
    }
    
    println!("\nPart 1 Total: {}", total1);
    
    // Part 2
    println!("\n=== Part 2 ===");
    let machines2 = parse_input("assets/day10machines2.txt")?;
    let num_machines2 = machines2.len();
    println!("Parsed {} machines", num_machines2);
    
    let mut total2 = 0;
    for (i, machine) in machines2.into_iter().enumerate() {
        let presses = solve_joltage(&machine);
        if (i + 1) % 10 == 0 || i == num_machines2 - 1 {
            println!("Machine {}: {} presses", i + 1, presses);
        }
        total2 += presses;
    }
    
    println!("\nPart 2 Total: {}", total2);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1_joltage_solution() {
        let machines = parse_input("assets/day10machines1.txt")
            .expect("Failed to load part 1 input");

        let mut total = 0;
        for (i, machine) in machines.iter().enumerate() {
            let presses = solve_joltage(&machine);
            println!("Machine {}: {} presses", i + 1, presses);
            total += presses;
        }

        assert_eq!(total, 33, "Part 1 joltage solution should be 33");
    }

    #[test]
    fn test_part2_joltage_solution() {
        let machines = parse_input("assets/day10machines2.txt")
            .expect("Failed to load part 2 input");

        let mut total = 0;
        for machine in machines.iter() {
            let presses = solve_joltage(&machine);
            total += presses;
        }

        assert_eq!(total, 17133, "Part 2 joltage solution should be 17133");
    }
}

