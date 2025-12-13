use anyhow::{anyhow, Context, Result};
use std::fs;
use std::collections::{HashMap, HashSet};
use varisat::{ExtendFormula, CnfFormula, Var, Lit, Solver};

#[derive(Debug, Clone)]
pub struct Shape {
    pub id: usize,
    pub grid: Vec<Vec<char>>, // 3x3 grid
}

#[derive(Debug, Clone)]
pub struct ProblemSpace {
    pub width: usize,
    pub height: usize, // "long" dimension
    pub shape_counts: Vec<usize>, // Count for each shape ID (index = shape ID)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Placement {
    pub shape_id: usize,
    pub instance: usize,
    pub x: i32,
    pub y: i32,
    pub cells: Vec<Coords>, // Actual grid cells occupied by this placement
}

fn parse_input(filename: &str) -> Result<(Vec<Shape>, Vec<ProblemSpace>)> {
    let content = fs::read_to_string(filename)
        .context(format!("Failed to read file: {}", filename))?;

    // Collect all lines but trim trailing empty lines
    let all_lines: Vec<&str> = content.lines().collect();
    let lines: Vec<&str> = all_lines.iter()
        .copied()
        .rev()
        .skip_while(|line| line.trim().is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let mut shapes = Vec::new();
    let mut spaces = Vec::new();
    let mut i = 0;

    // Parse shapes first
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Check if this is a shape definition (ID:)
        if line.ends_with(':') && !line.contains('x') {
            // Extract shape ID
            let id_str = &line[..line.len() - 1];
            let id = id_str.parse::<usize>()
                .context(format!("Line {}: invalid shape ID '{}'", i + 1, id_str))?;
            
            // Read the next 3 lines as the 3x3 grid
            if i + 3 >= lines.len() {
                return Err(anyhow!("Line {}: shape {} incomplete, expected 3 grid lines", i + 1, id));
            }
            
            let mut grid = Vec::new();
            for j in 1..=3 {
                let grid_line = lines[i + j].trim();
                if grid_line.len() != 3 {
                    return Err(anyhow!(
                        "Line {}: shape {} grid line {} should be 3 characters, got '{}'",
                        i + j + 1, id, j, grid_line
                    ));
                }
                grid.push(grid_line.chars().collect());
            }
            
            shapes.push(Shape { id, grid });
            i += 4; // Skip ID line and 3 grid lines, plus empty line if present
        } else if line.contains('x') && line.contains(':') {
            // This is a problem space definition
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Line {}: invalid problem space format", i + 1));
            }
            
            // Parse dimensions (e.g., "12x5")
            let dims: Vec<&str> = parts[0].trim().split('x').collect();
            if dims.len() != 2 {
                return Err(anyhow!("Line {}: invalid dimensions format, expected 'WxH'", i + 1));
            }
            
            let width = dims[0].parse::<usize>()
                .context(format!("Line {}: invalid width '{}'", i + 1, dims[0]))?;
            let height = dims[1].parse::<usize>()
                .context(format!("Line {}: invalid height '{}'", i + 1, dims[1]))?;
            
            // Parse shape counts
            let counts_str = parts[1].trim();
            let shape_counts: Vec<usize> = counts_str
                .split_whitespace()
                .map(|s| {
                    s.parse::<usize>()
                        .context(format!("Line {}: invalid shape count '{}'", i + 1, s))
                })
                .collect::<Result<Vec<_>>>()?;
            
            spaces.push(ProblemSpace {
                width,
                height,
                shape_counts,
            });
            i += 1;
        } else if line.is_empty() {
            // Skip empty lines
            i += 1;
        } else {
            // Unexpected line format
            return Err(anyhow!("Line {}: unexpected format '{}'", i + 1, line));
        }
    }

    Ok((shapes, spaces))
}

impl Shape {
    fn get_cells(&self) -> Vec<Coords> {
        let mut cells = Vec::new();
        for (y, row) in self.grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                if ch == '#' {
                    cells.push(Coords { x: x as i32, y: y as i32 });
                }
            }
        }
        cells
    }

    fn rotate_90(cells: &[Coords]) -> Vec<Coords> {
        cells.iter().map(|c| Coords { x: -c.y, y: c.x }).collect()
    }

    fn flip_horizontal(cells: &[Coords]) -> Vec<Coords> {
        cells.iter().map(|c| Coords { x: -c.x, y: c.y }).collect()
    }

    fn normalize(cells: &[Coords]) -> Vec<Coords> {
        if cells.is_empty() {
            return Vec::new();
        }
        let min_x = cells.iter().map(|c| c.x).min().unwrap();
        let min_y = cells.iter().map(|c| c.y).min().unwrap();
        let mut normalized: Vec<Coords> = cells
            .iter()
            .map(|c| Coords { x: c.x - min_x, y: c.y - min_y })
            .collect();
        normalized.sort_by_key(|c| (c.y, c.x));
        normalized
    }

    fn get_unique_transformations(&self) -> Vec<Vec<Coords>> {
        let base_cells = self.get_cells();
        let mut transformations = HashSet::new();

        // Try all 4 rotations
        let mut current = base_cells.clone();
        for _ in 0..4 {
            transformations.insert(Self::normalize(&current));
            current = Self::rotate_90(&current);
        }

        // Try flipped + 4 rotations
        let flipped = Self::flip_horizontal(&base_cells);
        let mut current = flipped;
        for _ in 0..4 {
            transformations.insert(Self::normalize(&current));
            current = Self::rotate_90(&current);
        }

        // HashSet automatically deduplicates, so symmetric shapes
        // will have fewer transformations
        transformations.into_iter().collect()
    }

    fn count_cells(&self) -> usize {
        self.grid.iter()
            .flat_map(|row| row.iter())
            .filter(|&&ch| ch == '#')
            .count()
    }
}

fn generate_placements(
    shape: &Shape,
    instance: usize,
    width: usize,
    height: usize,
) -> Vec<Placement> {
    let mut placements = Vec::new();
    let transformations = shape.get_unique_transformations();

    for transform in &transformations {
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let cells: Vec<Coords> = transform
                    .iter()
                    .map(|c| Coords { x: x + c.x, y: y + c.y })
                    .collect();

                if cells.iter().all(|c| c.x >= 0 && c.x < width as i32 && c.y >= 0 && c.y < height as i32) {
                    placements.push(Placement {
                        shape_id: shape.id,
                        instance,
                        x,
                        y,
                        cells,
                    });
                }
            }
        }
    }

    placements
}

fn solve_with_sat(
    shapes: &[Shape],
    space: &ProblemSpace,
) -> Result<Option<Vec<Placement>>> {
    solve_with_sat_verbose(shapes, space, false)
}

fn solve_with_sat_verbose(
    shapes: &[Shape],
    space: &ProblemSpace,
    verbose: bool,
) -> Result<Option<Vec<Placement>>> {
    let mut all_placements = Vec::new();
    let mut placement_to_var = HashMap::new();
    let mut var_to_placement = HashMap::new();
    let mut next_var = 1usize;

    let total_pieces: usize = space.shape_counts.iter().sum();
    if verbose {
        println!("Generating placements for {} total pieces...", total_pieces);
    }

    for (shape_idx, &count) in space.shape_counts.iter().enumerate() {
        if count == 0 {
            continue;
        }

        let shape = shapes.iter().find(|s| s.id == shape_idx)
            .ok_or_else(|| anyhow!("Shape {} not found", shape_idx))?;

        for instance in 0..count {
            let placements = generate_placements(shape, instance, space.width, space.height);
            if verbose {
                println!("  Shape {} instance {}: {} possible placements", shape_idx, instance, placements.len());
            }

            for placement in placements {
                let var = Var::from_index(next_var);
                next_var += 1;
                placement_to_var.insert(placement.clone(), var);
                var_to_placement.insert(var, placement.clone());
                all_placements.push(placement);
            }
        }
    }

    if verbose {
        println!("Total placements (variables): {}", all_placements.len());
    }

    let mut formula = CnfFormula::new();

    for (shape_idx, &count) in space.shape_counts.iter().enumerate() {
        if count == 0 {
            continue;
        }

        for instance in 0..count {
            let instance_placements: Vec<&Placement> = all_placements
                .iter()
                .filter(|p| p.shape_id == shape_idx && p.instance == instance)
                .collect();

            let vars: Vec<Lit> = instance_placements
                .iter()
                .map(|p| placement_to_var[p].positive())
                .collect();

            formula.add_clause(&vars);

            for i in 0..vars.len() {
                for j in i + 1..vars.len() {
                    formula.add_clause(&[!vars[i], !vars[j]]);
                }
            }
        }
    }

    let mut cell_to_placements: HashMap<Coords, Vec<Var>> = HashMap::new();
    for (placement, &var) in &placement_to_var {
        for &cell in &placement.cells {
            cell_to_placements.entry(cell).or_insert_with(Vec::new).push(var);
        }
    }

    if verbose {
        println!("Encoding grid cell constraints...");
    }
    for (_cell, vars) in &cell_to_placements {
        for i in 0..vars.len() {
            for j in i + 1..vars.len() {
                formula.add_clause(&[!vars[i].positive(), !vars[j].positive()]);
            }
        }
    }

    if verbose {
        println!("Solving SAT problem with {} variables and {} clauses...", next_var - 1, formula.len());
    }

    let mut solver = Solver::new();
    solver.add_formula(&formula);

    if solver.solve().unwrap() {
        if verbose {
            println!("Solution found!");
        }
        let model = solver.model().unwrap();
        let solution: Vec<Placement> = model
            .iter()
            .filter_map(|&lit| {
                if lit.is_positive() {
                    var_to_placement.get(&lit.var()).cloned()
                } else {
                    None
                }
            })
            .collect();

        Ok(Some(solution))
    } else {
        if verbose {
            println!("No solution exists");
        }
        Ok(None)
    }
}

fn visualize_solution(solution: &[Placement], width: usize, height: usize) {
    let mut grid = vec![vec!['.'; width]; height];

    for placement in solution {
        let symbol = (b'0' + placement.shape_id as u8) as char;
        for cell in &placement.cells {
            grid[cell.y as usize][cell.x as usize] = symbol;
        }
    }

    for row in grid {
        println!("{}", row.iter().collect::<String>());
    }
}

fn solve_with_backtracking(
    shapes: &[Shape],
    space: &ProblemSpace,
) -> Result<Option<Vec<Placement>>> {
    let width = space.width;
    let height = space.height;
    let mut grid = vec![vec![None; width]; height];

    let mut pieces_to_place = Vec::new();
    for (shape_idx, &count) in space.shape_counts.iter().enumerate() {
        for instance in 0..count {
            let shape = shapes.iter().find(|s| s.id == shape_idx)
                .ok_or_else(|| anyhow!("Shape {} not found", shape_idx))?;

            pieces_to_place.push((shape_idx, instance, shape.clone()));
        }
    }

    // Sort by most constrained first (fewest unique transformations, then largest size)
    pieces_to_place.sort_by_key(|(_, _, shape)| {
        let num_transforms = shape.get_unique_transformations().len();
        let num_cells = shape.count_cells();
        // Prioritize: fewest transformations first, then most cells
        (num_transforms, -(num_cells as i32))
    });

    let mut solution = Vec::new();

    if backtrack_optimized(
        &pieces_to_place,
        0,
        &mut grid,
        width,
        height,
        &mut solution,
        shapes,
    ) {
        Ok(Some(solution))
    } else {
        Ok(None)
    }
}

fn can_place_cells(cells: &[Coords], grid: &[Vec<Option<usize>>]) -> bool {
    cells.iter().all(|c| grid[c.y as usize][c.x as usize].is_none())
}

fn place_cells(cells: &[Coords], grid: &mut [Vec<Option<usize>>], piece_id: usize) {
    for cell in cells {
        grid[cell.y as usize][cell.x as usize] = Some(piece_id);
    }
}

fn remove_cells(cells: &[Coords], grid: &mut [Vec<Option<usize>>]) {
    for cell in cells {
        grid[cell.y as usize][cell.x as usize] = None;
    }
}

fn find_first_empty(grid: &[Vec<Option<usize>>], width: usize, height: usize) -> Option<(usize, usize)> {
    for y in 0..height {
        for x in 0..width {
            if grid[y][x].is_none() {
                return Some((x, y));
            }
        }
    }
    None
}

fn count_empty_cells(grid: &[Vec<Option<usize>>]) -> usize {
    grid.iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.is_none())
        .count()
}

fn count_remaining_cells(pieces: &[(usize, usize, Shape)], start_idx: usize) -> usize {
    pieces[start_idx..]
        .iter()
        .map(|(_, _, shape)| shape.count_cells())
        .sum()
}

fn backtrack_optimized(
    pieces: &[(usize, usize, Shape)],
    piece_idx: usize,
    grid: &mut [Vec<Option<usize>>],
    width: usize,
    height: usize,
    solution: &mut Vec<Placement>,
    _shapes: &[Shape],
) -> bool {
    if piece_idx == pieces.len() {
        return true;
    }

    // Early failure detection: check if we have enough space for remaining pieces
    let empty_cells = count_empty_cells(grid);
    let remaining_cells = count_remaining_cells(pieces, piece_idx);

    if empty_cells < remaining_cells {
        // Not enough space - prune this branch
        return false;
    }

    let (shape_id, instance, shape) = &pieces[piece_idx];

    let transformations = shape.get_unique_transformations();

    for transform in &transformations {
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let cells: Vec<Coords> = transform
                    .iter()
                    .map(|c| Coords { x: x + c.x, y: y + c.y })
                    .collect();

                if cells.iter().all(|c| {
                    c.x >= 0 && c.x < width as i32 &&
                    c.y >= 0 && c.y < height as i32
                }) && can_place_cells(&cells, grid) {
                    let placement = Placement {
                        shape_id: *shape_id,
                        instance: *instance,
                        x,
                        y,
                        cells: cells.clone(),
                    };

                    place_cells(&cells, grid, piece_idx);
                    solution.push(placement);

                    if backtrack_optimized(pieces, piece_idx + 1, grid, width, height, solution, _shapes) {
                        return true;
                    }

                    solution.pop();
                    remove_cells(&cells, grid);
                }
            }
        }
    }

    false
}

fn solve_part(filename: &str, part_name: &str, show_visualizations: bool) -> Result<usize> {
    let (shapes, spaces) = parse_input(filename)?;

    println!("\n========== {} ==========", part_name);
    println!("Parsed {} shapes", shapes.len());
    println!("Parsed {} problem spaces", spaces.len());

    let mut solution_count = 0;

    for (i, space) in spaces.iter().enumerate() {
        if show_visualizations {
            println!("\n----- Problem Space {} -----", i + 1);
            println!("Dimensions: {}x{}", space.width, space.height);
            println!("Shape counts: {:?}", space.shape_counts);
        } else {
            print!("\rSolving space {}/{} ({} solved so far)...", i + 1, spaces.len(), solution_count);
            use std::io::Write;
            std::io::stdout().flush().ok();
        }

        match solve_with_sat_verbose(&shapes, space, show_visualizations)? {
            Some(solution) => {
                solution_count += 1;
                if show_visualizations {
                    println!("\nSolution visualization:");
                    visualize_solution(&solution, space.width, space.height);
                }
            }
            None => {
                if show_visualizations {
                    println!("No solution found");
                }
            }
        }
    }

    if !show_visualizations {
        println!();
    }

    println!("\n{} Summary: {} / {} problem spaces solved", part_name, solution_count, spaces.len());

    Ok(solution_count)
}

fn solve_part_backtracking(filename: &str, part_name: &str, show_visualizations: bool) -> Result<usize> {
    let (shapes, spaces) = parse_input(filename)?;

    println!("\n========== {} (Backtracking) ==========", part_name);
    println!("Parsed {} shapes", shapes.len());
    println!("Parsed {} problem spaces", spaces.len());

    let mut solution_count = 0;

    for (i, space) in spaces.iter().enumerate() {
        if show_visualizations {
            println!("\n----- Problem Space {} -----", i + 1);
            println!("Dimensions: {}x{}", space.width, space.height);
            println!("Shape counts: {:?}", space.shape_counts);
        } else {
            print!("\rSolving space {}/{} ({} solved so far)...", i + 1, spaces.len(), solution_count);
            use std::io::Write;
            std::io::stdout().flush().ok();
        }

        match solve_with_backtracking(&shapes, space)? {
            Some(solution) => {
                solution_count += 1;
                if show_visualizations {
                    println!("\nSolution visualization:");
                    visualize_solution(&solution, space.width, space.height);
                }
            }
            None => {
                if show_visualizations {
                    println!("No solution found");
                }
            }
        }
    }

    if !show_visualizations {
        println!();
    }

    println!("\n{} Summary: {} / {} problem spaces solved", part_name, solution_count, spaces.len());

    Ok(solution_count)
}

/// Day 12: Exercise description
pub fn run() -> Result<()> {
    // Analyze shape symmetries
    let (shapes, spaces) = parse_input("assets/day12trees2.txt")?;
    println!("Analyzing shape symmetries for Part 2:");
    for shape in &shapes {
        let transformations = shape.get_unique_transformations();
        println!("  Shape {}: {} cells, {} unique transformations (out of 8 possible)",
            shape.id, shape.count_cells(), transformations.len());
    }

    println!("\n\nUsing SAT solver for Part 1 (small problems)...");
    solve_part("assets/day12trees1.txt", "Part 1", true)?;

    println!("\n\nSolving ALL Part 2 problems with backtracking + early pruning...");

    use std::time::Instant;
    let total_start = Instant::now();
    let mut solved = 0;
    let mut failed = 0;

    for (i, space) in spaces.iter().enumerate() {
        if (i + 1) % 100 == 0 || i < 10 {
            print!("\rProgress: {}/{} ({} solved, {} failed)", i + 1, spaces.len(), solved, failed);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }

        match solve_with_backtracking(&shapes, space) {
            Ok(Some(_)) => solved += 1,
            Ok(None) => failed += 1,
            Err(_) => failed += 1,
        }
    }

    println!("\n\n========== Part 2 Results ==========");
    println!("Total problems: {}", spaces.len());
    println!("Solved: {}", solved);
    println!("Failed: {}", failed);
    println!("Total time: {:.2}s", total_start.elapsed().as_secs_f64());
    if solved > 0 {
        println!("Average per solved problem: {:.4}s", total_start.elapsed().as_secs_f64() / solved as f64);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1_has_two_solutions() {
        let (shapes, spaces) = parse_input("assets/day12trees1.txt").unwrap();

        let mut solution_count = 0;

        for space in &spaces {
            if let Some(_solution) = solve_with_sat(&shapes, space).unwrap() {
                solution_count += 1;
            }
        }

        assert_eq!(solution_count, 2, "Part 1 should have exactly 2 solutions");
    }

    #[test]
    fn test_part2_has_481_solutions() {
        let (shapes, spaces) = parse_input("assets/day12trees2.txt").unwrap();

        let mut solution_count = 0;

        for space in &spaces {
            if let Some(_solution) = solve_with_backtracking(&shapes, space).unwrap() {
                solution_count += 1;
            }
        }

        assert_eq!(solution_count, 481, "Part 2 should have exactly 481 solutions");
    }
}
