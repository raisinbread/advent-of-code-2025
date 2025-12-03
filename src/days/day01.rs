// Constants for the dial mechanics
const DIAL_MIN: i32 = 0;
const DIAL_MAX: i32 = 99;
const DIAL_SIZE: i32 = 100;
const START_VALUE: i32 = 50;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = String;
    
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            _ => Err(format!("Invalid direction: {}", c)),
        }
    }
}

struct Safe {
    // Current position on the dial (0-99)
    dial_value: i32,
    // Number of times the dial stopped exactly on zero
    stops_on_zero: i32,
    // Total number of times the dial passed through zero
    visits_zero: i32,
}

impl Safe {
    fn new() -> Self {
        Safe { 
            dial_value: START_VALUE, 
            stops_on_zero: 0, 
            visits_zero: 0 
        }
    }

    fn rotate(&mut self, amount: i32, direction: Direction) {
        let before_value = self.dial_value;
        let before_zero_visits = self.visits_zero;
        let before_stops_on_zero = self.stops_on_zero;
        
        // How much the dial changes, even with large spins
        let net_change: i32 = amount % DIAL_SIZE;

        // Apply rotation using a multiplier for cleaner code
        let direction_multiplier = match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };
        self.dial_value += direction_multiplier * net_change;

        // Register full-round trips past zero
        self.visits_zero += amount / DIAL_SIZE;

        // If the net amount also causes a zero visit, count it
        // but only if we didn't start at zero
        if before_value != 0 && (self.dial_value < DIAL_MIN || self.dial_value > DIAL_MAX || self.dial_value == 0) {
            self.visits_zero += 1;
        }

        // Normalize the dial to 0-99 range
        self.dial_value = ((self.dial_value % DIAL_SIZE) + DIAL_SIZE) % DIAL_SIZE;

        // Check for landed-on-zero case
        if self.dial_value == 0 {
            self.stops_on_zero += 1;
        }
        
        println!("{} -> {:?}{} -> {}", before_value, direction, amount, self.dial_value);
        println!("Zero visits: {} -> {}", before_zero_visits, self.visits_zero);
        println!("Stops on zero: {} -> {}", before_stops_on_zero, self.stops_on_zero);
        println!("--------------------------------");
    }
}

/// Parse a turn string like "L5" or "R10" into a direction and amount
fn parse_turn(line: &str) -> Result<(Direction, i32), Box<dyn std::error::Error>> {
    let direction = line.chars().next()
        .ok_or("Empty line")?
        .try_into()?;
    let amount = line.get(1..)
        .ok_or("Invalid turn format")?
        .parse()?;
    Ok((direction, amount))
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut safe = Safe::new();
    let turns = std::fs::read_to_string("assets/day01turns.txt")?;

    for turn in turns.lines() {
        let (direction, amount) = parse_turn(turn)?;
        safe.rotate(amount, direction);
    }

    println!("Safe value: {}", safe.dial_value);
    println!("Zero hits: {}", safe.stops_on_zero);
    println!("Zero visits: {}", safe.visits_zero);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_turn_left() {
        let (dir, amt) = parse_turn("L5").unwrap();
        assert_eq!(amt, 5);
        matches!(dir, Direction::Left);
    }

    #[test]
    fn test_parse_turn_right() {
        let (dir, amt) = parse_turn("R10").unwrap();
        assert_eq!(amt, 10);
        matches!(dir, Direction::Right);
    }
    
    #[test]
    fn test_rotate_right_simple() {
        let mut safe = Safe::new();
        safe.rotate(10, Direction::Right);
        assert_eq!(safe.dial_value, 60);
    }

    #[test]
    fn test_rotate_left_simple() {
        let mut safe = Safe::new();
        safe.rotate(10, Direction::Left);
        assert_eq!(safe.dial_value, 40);
    }

    #[test]
    fn test_rotate_wraps_around() {
        let mut safe = Safe::new();
        safe.rotate(60, Direction::Right); // 50 + 60 = 110, wraps to 10
        assert_eq!(safe.dial_value, 10);
    }

    #[test]
    fn test_full_solution_visits_zero() {
        let mut safe = Safe::new();
        let turns = std::fs::read_to_string("assets/day01turns.txt")
            .expect("Failed to read input file");

        for turn in turns.lines() {
            let (direction, amount) = parse_turn(turn).unwrap();
            safe.rotate(amount, direction);
        }

        assert_eq!(safe.visits_zero, 6789);
    }
}
