use anyhow::Result;
use std::fmt;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq)]
enum PositionState {
    Initial,
    Empty,
    Unmovable,
    Movable,
}

impl fmt::Debug for PositionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionState::Initial => write!(f, "?"),
            PositionState::Empty => write!(f, "."),
            PositionState::Unmovable => write!(f, "@"),
            PositionState::Movable => write!(f, "x"),
        }
    }
}

struct Lot {
    positions: Vec<Vec<PositionState>>,
}

impl Lot {
    // The 8 neighbor offsets (cardinal and diagonal directions)
    const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1),
    ];
    
    fn new() -> Self {
        Lot {
            positions: Vec::new(),
        }
    }
    
    /// Get all movable positions in the lot
    fn get_movable(&self) -> Vec<(usize, usize)> {
        let mut movable = Vec::new();
        for (row_idx, row) in self.positions.iter().enumerate() {
            for (col_idx, &state) in row.iter().enumerate() {
                if matches!(state, PositionState::Movable) {
                    movable.push((row_idx, col_idx));
                }
            }
        }
        movable
    }
    
    /// Count the number of movable positions in the lot
    fn count_movable(&self) -> u32 {
        self.get_movable().len() as u32
    }
    
    /// Check if changing from old_state to new_state should trigger neighbor updates
    fn should_update_neighbors(old_state: PositionState, new_state: PositionState) -> bool {
        if old_state == new_state {
            return false;
        }
        // Skip updates only for Initial -> Empty transitions (during initialization)
        // For all other state changes, update neighbors
        !matches!(
            (old_state, new_state),
            (PositionState::Initial, PositionState::Empty)
        )
    }
    
    fn add_position(&mut self, row: usize, col: usize, is_empty: bool) {
        // Ensure the grid is large enough to place the new position
        while self.positions.len() <= row {
            self.positions.push(Vec::new());
        }
        while self.positions[row].len() <= col {
            self.positions[row].push(PositionState::Initial);
        }
        
        let old_state = PositionState::Initial;
        
        // Determine what the starting state should be
        let new_state = if is_empty {
            PositionState::Empty
        } else {
            // For non-empty positions, determine Movable/Unmovable based on neighbors
            Self::determine_state(self, row, col)
        };
        
        // Place it in the lot BEFORE updating neighbors
        // This ensures neighbors can see the correct state when they check this position
        self.positions[row][col] = new_state;
        
        // Update neighbors if needed
        if Self::should_update_neighbors(old_state, new_state) {
            self.update_neighbors_at(row, col);
        }
    }
    
    /// Remove a roll at a position by setting it to Empty.
    /// Only works if the position is currently Movable.
    /// Returns an error if position doesn't exist or isn't Movable.
    pub fn remove_roll_at(&mut self, row: usize, col: usize) -> Result<()> {
        // Check bounds
        if row >= self.positions.len() || col >= self.positions[row].len() {
            return Err(anyhow::anyhow!("Position ({}, {}) does not exist", row, col));
        }
        
        let old_state = self.positions[row][col];
        
        // Check if position is Movable
        if !matches!(old_state, PositionState::Movable) {
            return Err(anyhow::anyhow!(
                "Position ({}, {}) is {:?}, not Movable",
                row, col, old_state
            ));
        }
        
        // Set the position to Empty
        let new_state = PositionState::Empty;
        self.positions[row][col] = new_state;
        
        // Update neighbors if needed
        if Self::should_update_neighbors(old_state, new_state) {
            self.update_neighbors_at(row, col);
        }
        
        Ok(())
    }
    
    /// Count non-empty neighbors for a position at (row, col)
    fn count_non_empty_neighbors(lot: &Lot, row: usize, col: usize) -> usize {
        let mut count = 0;
        for (row_offset, col_offset) in Self::NEIGHBOR_OFFSETS {
            let neighbor_row = row as i32 + row_offset;
            let neighbor_col = col as i32 + col_offset;
            
            if neighbor_row >= 0 && neighbor_col >= 0 {
                let neighbor_row = neighbor_row as usize;
                let neighbor_col = neighbor_col as usize;
                
                if neighbor_row < lot.positions.len() && neighbor_col < lot.positions[neighbor_row].len() {
                    match lot.positions[neighbor_row][neighbor_col] {
                        PositionState::Initial | PositionState::Empty => {},
                        PositionState::Unmovable | PositionState::Movable => count += 1,
                    }
                }
            }
        }
        count
    }
    
    /// Determine the state for a position based on its neighbors
    pub(crate) fn determine_state(lot: &Lot, row: usize, col: usize) -> PositionState {
        // If position is Empty, it stays Empty regardless of neighbors
        if row < lot.positions.len() && 
           col < lot.positions[row].len() && 
           matches!(lot.positions[row][col], PositionState::Empty) {
            return PositionState::Empty;
        }
        
        // For non-empty positions, determine Movable/Unmovable based on neighbors
        let non_empty_count = Self::count_non_empty_neighbors(lot, row, col);
        if non_empty_count < 4 {
            PositionState::Movable
        } else {
            PositionState::Unmovable
        }
    }
    
    /// Update all 8 neighbors of a position at (row, col)
    /// Uses iterative processing to avoid recursive borrowing issues
    fn update_neighbors_at(&mut self, row: usize, col: usize) {
        // Use a queue to process all neighbor updates iteratively
        let mut queue = vec![(row, col)];
        let mut processed = HashSet::new();
        
        while let Some((current_row, current_col)) = queue.pop() {
            if processed.contains(&(current_row, current_col)) {
                continue;
            }
            processed.insert((current_row, current_col));
            
            // Collect neighbor updates for this position
            let mut updates = Vec::new();
            
            for (row_offset, col_offset) in Self::NEIGHBOR_OFFSETS {
                let neighbor_row = current_row as i32 + row_offset;
                let neighbor_col = current_col as i32 + col_offset;
                
                if neighbor_row >= 0 && neighbor_col >= 0 {
                    let neighbor_row = neighbor_row as usize;
                    let neighbor_col = neighbor_col as usize;
                    
                    // Ensure neighbor exists
                    if neighbor_row >= self.positions.len() || 
                       neighbor_col >= self.positions[neighbor_row].len() {
                        continue;
                    }
                    
                    let current_state = self.positions[neighbor_row][neighbor_col];
                    
                    // Skip if neighbor is Initial or Empty
                    if matches!(current_state, PositionState::Initial | PositionState::Empty) {
                        continue;
                    }
                    
                    // Determine new state for neighbor
                    let new_state = Self::determine_state(self, neighbor_row, neighbor_col);
                    
                    // Check if state actually needs to change
                    if current_state != new_state {
                        updates.push((neighbor_row, neighbor_col, current_state, new_state));
                    }
                }
            }
            
            // Apply updates and collect further neighbor updates
            for (neighbor_row, neighbor_col, old_state, new_state) in updates {
                self.positions[neighbor_row][neighbor_col] = new_state;
                
                if Self::should_update_neighbors(old_state, new_state) && 
                   !processed.contains(&(neighbor_row, neighbor_col)) {
                    queue.push((neighbor_row, neighbor_col));
                }
            }
        }
    }
    
}

impl fmt::Debug for Lot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Lot (movable: {})", self.count_movable())?;
        for row in &self.positions {
            for &state in row {
                write!(f, "{:?}", state)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Day 4: Exercise description
pub fn run() -> Result<()> {
    let input = std::fs::read_to_string("assets/day04rolls.txt")?;
    
    let mut lot = Lot::new();
    
    // Build the initial lot from the input file
    for (row, line) in input.lines().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let is_empty = match ch {
                '.' => true,
                '@' => false,
                _ => {
                    eprintln!("Warning: Unexpected character '{}' at row {}, col {}, treating as empty", ch, row, col);
                    true
                }
            };
            lot.add_position(row, col, is_empty);
        }
    }
    
    println!("Initial lot:");
    println!("{:?}", lot);
    println!();
    
    let mut total_removed = 0;
    let mut stage = 1;
    
    loop {
        // Get all currently movable positions
        let movable_positions = lot.get_movable();
        
        if movable_positions.is_empty() {
            break;
        }
        
        // Remove rolls at all movable positions
        let removed_count = movable_positions.len();
        for (row, col) in movable_positions {
            lot.remove_roll_at(row, col)?;
        }
        
        total_removed += removed_count;
        
        println!("Stage {}:", stage);
        println!("  Removed {} rolls", removed_count);
        println!("  Total removed so far: {}", total_removed);
        println!("{:?}", lot);
        println!();
        
        stage += 1;
    }
    
    println!("Final result:");
    println!("  Total stages: {}", stage - 1);
    println!("  Total rolls removed: {}", total_removed);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_solution_lot_count() {
        // Ensure the solution to part 1 stays correct.
        let input = std::fs::read_to_string("assets/day04rolls.txt")
            .expect("Failed to read input file");
        
        let mut lot = Lot::new();
        
        for (row, line) in input.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let is_empty = match ch {
                    '.' => true,
                    '@' => false,
                    _ => true,
                };
                lot.add_position(row, col, is_empty);
            }
        }
        
        assert_eq!(lot.count_movable(), 1433);
    }

    #[test]
    fn test_full_solution_total_removed() {
        // Ensure the solution to part 2 stays correct.
        let input = std::fs::read_to_string("assets/day04rolls.txt")
            .expect("Failed to read input file");
        
        let mut lot = Lot::new();
        
        for (row, line) in input.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let is_empty = match ch {
                    '.' => true,
                    '@' => false,
                    _ => true,
                };
                lot.add_position(row, col, is_empty);
            }
        }
        
        let mut total_removed = 0;
        
        loop {
            let movable_positions = lot.get_movable();
            
            if movable_positions.is_empty() {
                break;
            }
            
            let removed_count = movable_positions.len();
            for (row, col) in movable_positions {
                lot.remove_roll_at(row, col).expect("Failed to remove roll");
            }
            
            total_removed += removed_count;
        }
        
        assert_eq!(total_removed, 8616);
    }
}
