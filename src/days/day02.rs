use anyhow::{anyhow, Result};

#[derive(Clone, Copy)]
enum RepeatMode {
    ExactlyTwice,
    AnyCount,
}

fn parse_ranges(line: &str) -> Result<Vec<(&str, &str)>> {
    line.split(',')
        .map(|range| {
            range.split_once('-').ok_or_else(|| anyhow!("Invalid range format: {}", range))
        })
        .collect()
}

fn is_invalid_id(id: u128, repeat_mode: RepeatMode) -> bool {
    let s = id.to_string();

    match repeat_mode {
        RepeatMode::ExactlyTwice => {
            if !s.len().is_multiple_of(2) {
                return false;
            }
            let (first, second) = s.split_at(s.len() / 2);
            first == second
        }
        RepeatMode::AnyCount => {
        let mut num_parts = 2;
        while s.len() / num_parts >= 1 {
            if !s.len().is_multiple_of(num_parts) {
                num_parts += 1;
                continue;
            }
            let chunk_size = s.len() / num_parts;

            // Get the slices of the string.
            let chunks: Vec<&str> = (0..num_parts)
                .map(|i| &s[i * chunk_size..(i + 1) * chunk_size])
                .collect();

            // Compare adjacent chunks to see if they match.
            if chunks.windows(2).all(|w| w[0] == w[1]) {
                return true;
            }
                num_parts += 1;
            }
            false
        }
    }
}

fn find_invalid_ids_in_range(range: (&str, &str), repeat_mode: RepeatMode) -> Result<Vec<u128>, Box<dyn std::error::Error>> {
    if range.1.len() == 1 {
        return Ok(vec![]);
    }

    let start: u128 = range.0.parse()?;
    let end: u128 = range.1.parse()?;

    Ok((start..=end)
        .filter(|&id| is_invalid_id(id, repeat_mode))
        .collect())
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string("assets/day02ranges.txt")?;
    let ranges = parse_ranges(input.trim())?;

    let do_exactly_twice = false;

    if do_exactly_twice {
        let mut double_repeat_invalid_ids: Vec<u128> = Vec::new();
        for range in ranges {
            let invalid_ids = find_invalid_ids_in_range(range, RepeatMode::ExactlyTwice)?;
            double_repeat_invalid_ids.extend(invalid_ids);
        }

        let sum: u128 = double_repeat_invalid_ids.iter().sum();
        println!("{:?}", double_repeat_invalid_ids);
        println!("Sum: {}", sum);
    } else {
        let mut any_repeat_invalid_ids: Vec<u128> = Vec::new();
        for range in ranges {
            let invalid_ids = find_invalid_ids_in_range(range, RepeatMode::AnyCount)?;
            any_repeat_invalid_ids.extend(invalid_ids);
        }

        let sum: u128 = any_repeat_invalid_ids.iter().sum();
        println!("{:?}", any_repeat_invalid_ids);
        println!("Sum: {}", sum);
    }

    Ok(())
}

