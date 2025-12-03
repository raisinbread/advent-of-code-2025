use anyhow::{anyhow, Result};

// Parse a line of digits into a vector of integers
fn parse_bank_line(line: &str) -> Result<Vec<u32>> {
    line.chars()
        .map(|c| {
            c.to_digit(10)
                .ok_or_else(|| anyhow!("Invalid digit: {}", c))
        })
        .collect()
}

// Parse the banks file, returning a vector of vectors (one per line)
fn parse_banks_file(file_path: &str) -> Result<Vec<Vec<u32>>> {
    let contents = std::fs::read_to_string(file_path)?;
    contents
        .lines()
        .map(|line| parse_bank_line(line.trim()))
        .collect()
}

fn find_largest_joltage_settings(bank: &[u32], n: usize) -> Result<u64> {
    // Validate that n is not greater than bank size
    if n > bank.len() {
        return Err(anyhow!("n ({}) must be <= bank size ({})", n, bank.len()));
    }
    
    if n == 0 {
        return Ok(0);
    }

    // Dynamic programming approach: dp[position][digits_used] = max number we can form
    let mut dp = vec![vec![None; n + 1]; bank.len()];
    
    // Base case 1: using 0 digits gives 0
    for row in &mut dp {
        row[0] = Some(0u64);
    }
    
    // Base case 2: using 1 digit from position 0
    dp[0][1] = Some(bank[0] as u64);
    
    // Fill the DP table
    for i in 1..bank.len() {
        let digit = bank[i] as u64;

        for j in 1..=n.min(i + 1) {
            // Option 1: Don't use digit at position i
            let option1 = dp[i - 1][j];

            // Option 2: Use digit at position i
            let option2 = dp[i - 1][j - 1].map(|prev| prev * 10 + digit);

            // Take the maximum of both options
            dp[i][j] = option1.into_iter().chain(option2).max();
        }
    }
    
    // The answer is dp[bank.len() - 1][n]
    dp[bank.len() - 1][n]
        .ok_or_else(|| anyhow!("Could not form a number with {} digits", n))
}

// Day 3: Exercise description
pub fn run() -> Result<()> {
    let banks = parse_banks_file("assets/day03banks.txt")?;
    
    let mut largest_settings = Vec::new();
    let do_only_two_batteries = false;
    
    for bank in &banks {
        // Print the values in the bank
        println!("Bank: {:?}", bank);
        
        // Find the largest setting for this bank (using 2 elements by default)
        let largest = find_largest_joltage_settings(bank, if do_only_two_batteries { 2 } else { 12 })?;
        println!("Largest setting: {}", largest);
        
        largest_settings.push(largest);
    }
    
    // Sum all the largest settings
    let sum: u64 = largest_settings.iter().sum();
    println!("\nFinal sum: {}", sum);
    
    Ok(())
}

