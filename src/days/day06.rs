use anyhow::{anyhow, Result};
use std::fs;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Multiply,
    Add,
}

impl FromStr for Operator {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "*" => Ok(Operator::Multiply),
            "+" => Ok(Operator::Add),
            _ => Err(anyhow!("Unknown operator: {}", s)),
        }
    }
}

impl Operator {
    fn apply(&self, a: i64, b: i64) -> i64 {
        match self {
            Operator::Multiply => a * b,
            Operator::Add => a + b,
        }
    }
}

fn parse_input(filename: &str) -> Result<(Vec<Vec<i64>>, Vec<Operator>)> {
    let content = fs::read_to_string(filename)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        return Err(anyhow!("Input file is empty"));
    }
    
    // Parse all lines except the last as integers
    let integer_lines = &lines[..lines.len() - 1];
    let grid: Vec<Vec<i64>> = integer_lines
        .iter()
        .map(|line| {
            line.split_whitespace()
                .map(|s| s.parse())
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    
    // Parse the last line as operators
    let operators: Vec<Operator> = lines[lines.len() - 1]
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<Vec<_>>>()?;
    
    Ok((grid, operators))
}

fn parse_input_col(filename: &str) -> Result<(Vec<Vec<Vec<char>>>, Vec<Operator>)> {
    let content = fs::read_to_string(filename)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.len() < 2 {
        return Err(anyhow!("Input file must have at least 2 lines"));
    }
    
    // Separate data lines from operator line
    let data_lines = &lines[..lines.len() - 1];
    let operator_line = lines[lines.len() - 1];
    
    if data_lines.is_empty() {
        return Err(anyhow!("No data lines found"));
    }
    
    // Parse each line to find number positions
    let mut all_number_positions: Vec<Vec<(usize, usize)>> = Vec::new(); // (start, end) for each number
    
    for line in data_lines.iter() {
        let chars: Vec<char> = line.chars().collect();
        let mut number_positions = Vec::new();
        let mut i = 0;
        
        while i < chars.len() {
            // Skip leading spaces
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
            
            if i >= chars.len() {
                break;
            }
            
            // Found the start of a number
            let start = i;
            
            // Find the end of the number
            while i < chars.len() && chars[i] != ' ' {
                i += 1;
            }
            
            let end = i;
            number_positions.push((start, end));
        }
        
        all_number_positions.push(number_positions);
    }
    
    // Determine number of columns
    let num_columns = all_number_positions.iter().map(|v| v.len()).max().unwrap_or(0);
    
    // For each column, find the leftmost start and rightmost end across all rows
    let mut column_starts = vec![usize::MAX; num_columns];
    let mut column_ends = vec![0; num_columns];
    
    for number_positions in &all_number_positions {
        for (col_idx, &(start, end)) in number_positions.iter().enumerate() {
            column_starts[col_idx] = column_starts[col_idx].min(start);
            column_ends[col_idx] = column_ends[col_idx].max(end);
        }
    }
    
    // Extract column data using these boundaries
    let mut columns = Vec::new();
    
    for col_idx in 0..num_columns {
        let start = column_starts[col_idx];
        let end = column_ends[col_idx];
        let mut column_data = Vec::new();
        
        for line in data_lines {
            let line_chars: Vec<char> = line.chars().collect();
            let mut row_chars = Vec::new();
            
            // Extract characters for this column
            for pos in start..end {
                if pos < line_chars.len() {
                    row_chars.push(line_chars[pos]);
                } else {
                    row_chars.push(' ');
                }
            }
            
            column_data.push(row_chars);
        }
        
        columns.push(column_data);
    }
    
    // Parse operators
    let operators: Vec<Operator> = operator_line
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<Vec<_>>>()?;
    
    Ok((columns, operators))
}

fn process_column(grid: &[Vec<i64>], col_idx: usize, operator: Operator) -> i64 {
    grid.iter()
        .map(|row| row[col_idx])
        .reduce(|acc, val| operator.apply(acc, val))
        .unwrap_or(0)
}

fn do_homework(grid: &[Vec<i64>], operators: &[Operator]) -> Result<Vec<i64>> {
    if grid.is_empty() {
        return Err(anyhow!("Grid is empty"));
    }
    
    let num_columns = grid[0].len();
    if operators.len() != num_columns {
        return Err(anyhow!(
            "Number of operators ({}) doesn't match number of columns ({})",
            operators.len(),
            num_columns
        ));
    }
    
    let results =
        operators
            .iter()
            .enumerate()
            .map(|(col_idx, &operator)| process_column(grid, col_idx, operator))
            .collect()
    ;
    
    Ok(results)
}

fn do_homework_col(columns: &[Vec<Vec<char>>], operators: &[Operator]) -> Result<Vec<i64>> {
    if columns.is_empty() {
        return Err(anyhow!("No columns provided"));
    }
    
    if operators.len() != columns.len() {
        return Err(anyhow!(
            "Number of operators ({}) doesn't match number of columns ({})",
            operators.len(),
            columns.len()
        ));
    }
    
    let mut results = Vec::new();
    
    for (col_idx, column) in columns.iter().enumerate() {
        let operator = operators[col_idx];
        
        if column.is_empty() {
            return Err(anyhow!("Column {} is empty", col_idx));
        }
        
        // Determine the width of this column (length of character arrays)
        let width = column[0].len();
        
        // For each character position, read down all rows to form a number
        let mut numbers = Vec::new();
        
        for char_pos in 0..width {
            let mut digit_string = String::new();
            
            // Read down all rows at this character position
            for row in column {
                if char_pos < row.len() {
                    let ch = row[char_pos];
                    if ch.is_ascii_digit() {
                        digit_string.push(ch);
                    }
                    // Skip non-digit characters (like spaces)
                }
            }
            
            // Convert to number (if we found any digits)
            if !digit_string.is_empty() {
                let number: i64 = digit_string.parse()
                    .map_err(|e| anyhow!("Failed to parse '{}': {}", digit_string, e))?;
                numbers.push(number);
            }
        }
        
        // Apply the operator across all numbers in this column
        let result = numbers
            .iter()
            .copied()
            .reduce(|acc, val| operator.apply(acc, val))
            .ok_or_else(|| anyhow!("No valid numbers found in column {}", col_idx))?;
        
        results.push(result);
    }
    
    Ok(results)
}

pub fn run() -> Result<()> {
    let (grid, operators) = parse_input("assets/day06problems.txt")?;
    
    println!("Day 6: Parsed {} lines of integers", grid.len());
    for (i, row) in grid.iter().enumerate() {
        println!("Line {}: {:?}", i, row);
    }
    
    println!("Operators: {:?}", operators);
    
    // Part 1: Standard mode
    let column_results = do_homework(&grid, &operators)?;
    let sum: i64 = column_results.iter().sum();
    println!("\nPart 1 (Standard mode):");
    println!("Column results: {:?}", column_results);
    println!("Sum: {}", sum);
    
    // Part 2: Column-based mode
    let (columns, col_operators) = parse_input_col("assets/day06problems.txt")?;
    println!("\n--- Part 2 (Column-based mode) ---");
    println!("Parsed {} columns", columns.len());
    
    // Show all columns
    // for (i, column) in columns.iter().enumerate() {
    //     println!("\nColumn {}:", i);
    //     for (row_idx, row_chars) in column.iter().enumerate() {
    //         println!("  Row {}: {:?}", row_idx, row_chars);
    //     }
    // }
    
    let col_results = do_homework_col(&columns, &col_operators)?;
    let col_sum: i64 = col_results.iter().sum();
    println!("\nColumn results: {:?}", col_results);
    println!("Sum: {}", col_sum);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_solution_part_one_sum() {
        let (grid, operators) = parse_input("assets/day06problems.txt")
            .expect("Failed to read input file");
        
        let column_results = do_homework(&grid, &operators)
            .expect("Failed to process homework");
        let sum: i64 = column_results.iter().sum();
        
        assert_eq!(sum, 4878670269096, "Part 1 final sum should be 4878670269096");
    }

    #[test]
    fn test_full_solution_part_two_sum() {
        let (columns, col_operators) = parse_input_col("assets/day06problems.txt")
            .expect("Failed to read input file");
        
        let col_results = do_homework_col(&columns, &col_operators)
            .expect("Failed to process column-based homework");
        let col_sum: i64 = col_results.iter().sum();
        
        assert_eq!(col_sum, 8674740488592, "Part 2 final sum should be 8674740488592");
    }
}
