use anyhow::{anyhow, Result};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IdRange {
    start: u64,
    end: u64,
}

impl IdRange {
    fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }
    
    fn contains(&self, id: u64) -> bool {
        id >= self.start && id <= self.end
    }
    
    fn overlaps_or_adjacent(&self, other: &IdRange) -> bool {
        other.start <= self.end + 1
    }
    
    fn merge(&self, other: &IdRange) -> IdRange {
        IdRange::new(self.start, self.end.max(other.end))
    }
    
    fn count(&self) -> u64 {
        self.end - self.start + 1
    }
}

pub fn run() -> Result<()> {
    let (ranges, ids) = parse_input("assets/day05ids.txt")?;
    println!("Day 5: Parsed {} ranges and {} IDs", ranges.len(), ids.len());
    
    let optimized_ranges = optimize_ranges(ranges);
    println!("Optimized to {} ranges", optimized_ranges.len());
    
    // Calculate total fresh IDs based on optimized ranges
    let total_fresh_from_ranges: u64 = optimized_ranges.iter()
        .map(|range| range.count())
        .sum();
    println!("Total fresh IDs from ranges: {}", total_fresh_from_ranges);
    
    // Check each ID to see if it's spoiled or fresh
    // Ranges represent FRESH IDs, so if ID is in range = fresh, otherwise = spoiled
    let fresh_count = ids.iter()
        .filter(|&&id| is_fresh(&optimized_ranges, id))
        .count();
    let spoiled_count = ids.len() - fresh_count;
    
    println!("\nResults:");
    println!("Spoiled IDs: {}", spoiled_count);
    println!("Fresh IDs: {}", fresh_count);
    
    Ok(())
}

fn is_fresh(ranges: &[IdRange], id: u64) -> bool {
    // Use binary search to check if id falls within any range
    // Ranges represent FRESH IDs (inclusive on both ends)
    // Ranges are sorted by start value and non-overlapping
    
    // Binary search for the rightmost range where start <= id
    let idx = match ranges.binary_search_by_key(&id, |range| range.start) {
        Ok(idx) => idx,  // Exact match on start
        Err(idx) => {
            if idx == 0 {
                return false;  // id is before all ranges
            }
            idx - 1  // Check the range just before
        }
    };
    
    // Check if id falls within the range at idx (inclusive on both ends)
    ranges[idx].contains(id)
}

fn optimize_ranges(mut ranges: Vec<IdRange>) -> Vec<IdRange> {
    if ranges.is_empty() {
        return ranges;
    }
    
    // Step 1: Sort by the start value (ascending)
    ranges.sort_by_key(|r| r.start);
    
    let mut optimized = Vec::new();
    let mut current = ranges[0];
    
    // Step 2-3: Inspect each element and its successor for overlaps or adjacency
    for &next in &ranges[1..] {
        // Check if current and next overlap or are adjacent
        if current.overlaps_or_adjacent(&next) {
            // They overlap or are adjacent, combine them
            current = current.merge(&next);
        } else {
            // No overlap or adjacency, save current and move to next
            optimized.push(current);
            current = next;
        }
    }
    
    // Don't forget to add the last range
    optimized.push(current);
    
    optimized
}

fn parse_input(filename: &str) -> Result<(Vec<IdRange>, Vec<u64>)> {
    let content = fs::read_to_string(filename)?;
    
    // Split the content by empty line
    let parts: Vec<&str> = content.split("\n\n").collect();
    if parts.len() < 2 {
        return Err(anyhow!("Input file must contain two sections separated by empty line"));
    }
    
    // Parse ranges (first part)
    let ranges: Vec<IdRange> = parts[0]
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let nums: Vec<u64> = line
                .split('-')
                .map(|n| n.trim().parse())
                .collect::<Result<_, _>>()?;
            if nums.len() != 2 {
                return Err(anyhow!("Invalid range format: {}", line));
            }
            Ok(IdRange::new(nums[0], nums[1]))
        })
        .collect::<Result<Vec<_>>>()?;
    
    // Parse IDs (second part)
    let ids: Vec<u64> = parts[1]
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().parse())
        .collect::<Result<_, _>>()?;
    
    Ok((ranges, ids))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_solution_parse_counts() {
        let (ranges, ids) = parse_input("assets/day05ids.txt")
            .expect("Failed to read input file");
        
        assert_eq!(ranges.len(), 183, "Should parse 183 ranges");
        assert_eq!(ids.len(), 1000, "Should parse 1000 IDs");
    }

    #[test]
    fn test_full_solution_optimized_ranges() {
        let (ranges, _) = parse_input("assets/day05ids.txt")
            .expect("Failed to read input file");
        
        let optimized_ranges = optimize_ranges(ranges);
        assert_eq!(optimized_ranges.len(), 78, "Should optimize to 78 ranges");
    }

    #[test]
    fn test_full_solution_total_fresh_ids() {
        let (ranges, _) = parse_input("assets/day05ids.txt")
            .expect("Failed to read input file");
        
        let optimized_ranges = optimize_ranges(ranges);
        let total_fresh: u64 = optimized_ranges.iter()
            .map(|range| range.count())
            .sum();
        
        assert_eq!(total_fresh, 369761800782619, "Total fresh IDs from ranges");
    }

    #[test]
    fn test_full_solution_spoiled_and_fresh_counts() {
        let (ranges, ids) = parse_input("assets/day05ids.txt")
            .expect("Failed to read input file");
        
        let optimized_ranges = optimize_ranges(ranges);
        
        let fresh_count = ids.iter()
            .filter(|&&id| is_fresh(&optimized_ranges, id))
            .count();
        let spoiled_count = ids.len() - fresh_count;
        
        assert_eq!(spoiled_count, 365, "Should have 365 spoiled IDs");
        assert_eq!(fresh_count, 635, "Should have 635 fresh IDs");
    }
}
