use clap::Parser;

mod days;

#[derive(Parser)]
#[command(name = "Advent of Code 2025")]
#[command(about = "Solutions for Advent of Code 2025", long_about = None)]
struct Cli {
    #[arg(value_parser = clap::value_parser!(u8).range(1..=12))]
    day: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    println!("🎄 Advent of Code 2025 - Day {} 🎄\n", cli.day);
    
    match cli.day {
        1 => days::day01::run()?,
        2 => days::day02::run()?,
        3 => days::day03::run()?,
        4 => days::day04::run()?,
        5 => days::day05::run()?,
        6 => days::day06::run(),
        7 => days::day07::run(),
        8 => days::day08::run(),
        9 => days::day09::run(),
        10 => days::day10::run(),
        11 => days::day11::run(),
        12 => days::day12::run(),
        _ => unreachable!("clap should prevent this"),
    }
    
    Ok(())
}
