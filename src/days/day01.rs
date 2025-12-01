enum Direction {
    Left,
    Right,
}

struct Safe {
    current_value: i32,
    zero_hits: i32
}

impl Safe {
    fn new() -> Self {
        Safe { current_value: 50, zero_hits: 0 }
    }

    fn rotate(&mut self, amount: i32, direction: Direction) {
        let before = self.current_value;
        let direction_str = match direction {
            Direction::Left => "L",
            Direction::Right => "R",
        };
        
        let net_change: i32 = amount % 100;
        match direction {
            Direction::Left => self.current_value -= net_change,
            Direction::Right => self.current_value += net_change,
        }
        self.current_value = ((self.current_value % 100) + 100) % 100;
        if self.current_value == 0 {
            self.zero_hits += 1;
        }
        println!("{} -> {}{} -> {}", before, direction_str, amount, self.current_value); 
    }
}

pub fn run() {
    let mut safe = Safe::new();
    let turns = std::fs::read_to_string("assets/day01turns.txt").unwrap();
    for turn in turns.lines() {
        let direction = match turn.chars().next().unwrap() {
            'L' => Direction::Left,
            'R' => Direction::Right,
            _ => panic!("Invalid direction"),
        };
        let amount = turn[1..].parse::<i32>().unwrap();
        safe.rotate(amount, direction);
    }
    println!("Safe value: {}", safe.current_value);
    println!("Zero hits: {}", safe.zero_hits);
}

