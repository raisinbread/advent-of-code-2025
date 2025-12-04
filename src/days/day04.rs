use anyhow::Result;
use std::fmt;
use std::collections::HashSet;

// Rc, for when multiple things hold a reference to the same thing
// RefCell, for when you want to mutably borrow something
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, PartialEq)]
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
    positions: Vec<Vec<Position>>,
}

impl Lot {
    // The 8 neighbor offsets (cardinal and diagonal directions)
    const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1),
    ];
    
    // Lots are reference-counted, and mutable-borrowable so they can be shared and updated by Positions
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Lot {
            positions: Vec::new(),
        }))
    }
    
    /// Count the number of movable positions in the lot
    fn count_movable(&self) -> u32 {
        self.positions.iter()
            .flat_map(|row| row.iter())
            .filter(|pos| matches!(pos.state, PositionState::Movable))
            .count() as u32
    }
    
    fn add_position(lot: &Rc<RefCell<Lot>>, row: usize, col: usize, is_empty: bool) {
        // Get a mutable borrow of the lot
        let mut lot_borrow = lot.borrow_mut();
        
        // Ensure the grid is large enough to place the new position
        while lot_borrow.positions.len() <= row {
            lot_borrow.positions.push(Vec::new());
        }
        while lot_borrow.positions[row].len() <= col {
            lot_borrow.positions[row].push(Position {
                state: PositionState::Initial,
            });
        }
        
        // 1. Create the position with Initial state
        let mut position = Position {
            state: PositionState::Initial,
        };
        
        // 2. Determine what the starting state should be
        let starting_state = if is_empty {
            PositionState::Empty
        } else {
            // For non-empty positions, determine Movable/Unmovable based on neighbors
            Self::determine_state(&lot_borrow, row, col)
        };
        
        // Release the borrow to allow set_state to update neighbors
        drop(lot_borrow);
        
        // Set the state
        let should_update_neighbors = position.set_state(starting_state);
        
        // 3. Place it in the lot BEFORE updating neighbors
        // This ensures neighbors can see the correct state when they check this position
        lot.borrow_mut().positions[row][col] = position;
        
        // Update neighbors if needed
        if should_update_neighbors {
            Self::update_neighbors_at(lot, row, col);
        }
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
                    match lot.positions[neighbor_row][neighbor_col].state {
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
           matches!(lot.positions[row][col].state, PositionState::Empty) {
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
    fn update_neighbors_at(lot: &Rc<RefCell<Lot>>, row: usize, col: usize) {
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
            
            {
                let lot_borrow = lot.borrow();
                
                for (row_offset, col_offset) in Self::NEIGHBOR_OFFSETS {
                    let neighbor_row = current_row as i32 + row_offset;
                    let neighbor_col = current_col as i32 + col_offset;
                    
                    if neighbor_row >= 0 && neighbor_col >= 0 {
                        let neighbor_row = neighbor_row as usize;
                        let neighbor_col = neighbor_col as usize;
                        
                        // Ensure neighbor exists
                        if neighbor_row >= lot_borrow.positions.len() || 
                           neighbor_col >= lot_borrow.positions[neighbor_row].len() {
                            continue;
                        }
                        
                        // Skip if neighbor is Initial or Empty
                        if matches!(
                            lot_borrow.positions[neighbor_row][neighbor_col].state,
                            PositionState::Initial | PositionState::Empty
                        ) {
                            continue;
                        }
                        
                        // Determine new state for neighbor
                        let new_state = Self::determine_state(&lot_borrow, neighbor_row, neighbor_col);
                        
                        // Check if state actually needs to change
                        let current_state = &lot_borrow.positions[neighbor_row][neighbor_col].state;
                        if *current_state != new_state {
                            updates.push((neighbor_row, neighbor_col, new_state));
                        }
                    }
                }
            } // lot_borrow is dropped here
            
            // Apply updates and collect further neighbor updates
            for (neighbor_row, neighbor_col, new_state) in updates {
                // Borrow mutably to call set_state, then drop the borrow immediately
                let should_update = {
                    let mut lot_borrow = lot.borrow_mut();
                    lot_borrow.positions[neighbor_row][neighbor_col]
                        .set_state(new_state)
                };
                
                if should_update && !processed.contains(&(neighbor_row, neighbor_col)) {
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
            for position in row {
                write!(f, "{:?}", position)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

struct Position {
    state: PositionState,
}

impl Position {
    /// Set the state of this position.
    /// Returns whether neighbors should be updated
    fn set_state(&mut self, new_state: PositionState) -> bool {
        let old_state = self.state.clone();
        
        if old_state != new_state {
            self.state = new_state.clone();
            
            // Check if neighbors should be updated
            // Skip updates for Initial -> Empty transitions
            !matches!(
                (old_state, new_state.clone()),
                (PositionState::Initial, PositionState::Empty)
            ) && matches!(
                new_state,
                PositionState::Movable | PositionState::Unmovable
            )
        } else {
            false
        }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.state)
    }
}

/// Day 4: Exercise description
pub fn run() -> Result<()> {
    let input = std::fs::read_to_string("assets/day04rolls.txt")?;
    
    let lot = Lot::new();
    
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
            Lot::add_position(&lot, row, col, is_empty);
        }
    }
    
    // 4. Debug print the lot (which includes its count)
    println!("{:?}", lot.borrow());
    
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
        
        let lot = Lot::new();
        
        for (row, line) in input.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let is_empty = match ch {
                    '.' => true,
                    '@' => false,
                    _ => true,
                };
                Lot::add_position(&lot, row, col, is_empty);
            }
        }
        
        assert_eq!(lot.borrow().count_movable(), 1433);
    }
}
