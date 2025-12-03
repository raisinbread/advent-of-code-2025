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
    let repeat_mode = if do_exactly_twice { RepeatMode::ExactlyTwice } else { RepeatMode::AnyCount };

    let mut invalid_ids: Vec<u128> = Vec::new();
    for range in ranges {
        invalid_ids.extend(find_invalid_ids_in_range(range, repeat_mode)?);
    }

    let sum: u128 = invalid_ids.iter().sum();
    println!("{:?}", invalid_ids);
    println!("Sum: {}", sum);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ranges() {
        let ranges = parse_ranges("10-20,30-40").unwrap();
        assert_eq!(ranges, vec![("10", "20"), ("30", "40")]);
    }

    #[test]
    fn test_is_invalid_id_exactly_twice() {
        // 1212 = "12" repeated twice
        assert!(is_invalid_id(1212, RepeatMode::ExactlyTwice));
        // 123123 = "123" repeated twice
        assert!(is_invalid_id(123123, RepeatMode::ExactlyTwice));

        // 123 - odd length, can't be exactly twice
        assert!(!is_invalid_id(123, RepeatMode::ExactlyTwice));
        // 1234 - not a repeat
        assert!(!is_invalid_id(1234, RepeatMode::ExactlyTwice));
        // 121212 - repeated 3 times, not exactly twice
        assert!(!is_invalid_id(121212, RepeatMode::ExactlyTwice));
    }

    #[test]
    fn test_is_invalid_id_any_count() {
        // 1212 = "12" repeated twice
        assert!(is_invalid_id(1212, RepeatMode::AnyCount));
        // 121212 = "12" repeated three times
        assert!(is_invalid_id(121212, RepeatMode::AnyCount));
        // 777 = "7" repeated three times
        assert!(is_invalid_id(777, RepeatMode::AnyCount));

        assert!(!is_invalid_id(1234, RepeatMode::AnyCount));
        assert!(!is_invalid_id(123, RepeatMode::AnyCount));
    }

    #[test]
    fn test_find_invalid_ids_in_range() {
        // Range 11-13 with AnyCount should find 11, 12 (no, 12 isn't repeating), 13 (no)
        // Actually 11 = "11" = "1" repeated twice
        let ids = find_invalid_ids_in_range(("11", "13"), RepeatMode::AnyCount).unwrap();
        assert!(ids.contains(&11));
        assert!(!ids.contains(&12));
        assert!(!ids.contains(&13));
    }

    #[test]
    fn test_full_solution_sum() {
        let input = std::fs::read_to_string("assets/day02ranges.txt")
            .expect("Failed to read input file");
        let ranges = parse_ranges(input.trim()).unwrap();

        let mut invalid_ids: Vec<u128> = Vec::new();
        for range in ranges {
            invalid_ids.extend(find_invalid_ids_in_range(range, RepeatMode::AnyCount).unwrap());
        }

        let sum: u128 = invalid_ids.iter().sum();
        assert_eq!(sum, 22471660255);
    }
}
