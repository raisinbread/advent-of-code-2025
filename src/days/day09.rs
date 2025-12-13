use anyhow::{anyhow, Context, Result};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate {
    x: usize,
    y: usize,
}

fn parse_input(filename: &str) -> Result<Vec<Coordinate>> {
    let content = fs::read_to_string(filename)
        .context(format!("Failed to read file: {}", filename))?;

    // Parse all coordinates
    let coordinates: Vec<Coordinate> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            let parts: Vec<&str> = line.trim().split(',').collect();
            if parts.len() != 2 {
                return Err(anyhow!(
                    "Line {} has {} values, expected 2 comma-separated values",
                    i + 1,
                    parts.len()
                ));
            }

            let x = parts[0].parse::<usize>()
                .context(format!("Failed to parse x coordinate on line {}", i + 1))?;
            let y = parts[1].parse::<usize>()
                .context(format!("Failed to parse y coordinate on line {}", i + 1))?;

            Ok(Coordinate { x, y })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(coordinates)
}

fn find_largest_rectangle(coordinates: &[Coordinate]) -> Option<Square> {
    if coordinates.len() < 2 {
        return None;
    }

    let mut largest_square: Option<Square> = None;

    // Check every pair of coordinates
    for i in 0..coordinates.len() {
        for j in (i + 1)..coordinates.len() {
            let coord1 = coordinates[i];
            let coord2 = coordinates[j];

            // Calculate distances
            let dx = if coord1.x > coord2.x {
                coord1.x - coord2.x
            } else {
                coord2.x - coord1.x
            };
            let dy = if coord1.y > coord2.y {
                coord1.y - coord2.y
            } else {
                coord2.y - coord1.y
            };

            // Both dimensions must be non-zero to form a rectangle
            if dx == 0 || dy == 0 {
                continue;
            }

            // Calculate area (add 1 to each dimension because coordinates are inclusive)
            let area = (dx + 1) * (dy + 1);

            // Update largest square if this one is bigger
            if largest_square.is_none() || area > largest_square.unwrap().area {
                largest_square = Some(Square {
                    corner1: coord1,
                    corner2: coord2,
                    area,
                });
            }
        }
    }

    largest_square
}

// Point-in-polygon test using ray casting algorithm
fn point_in_polygon(x: i64, y: i64, polygon: &[(i64, i64)]) -> bool {
    let mut inside = false;
    let n = polygon.len();

    for i in 0..n {
        let (x1, y1) = polygon[i];
        let (x2, y2) = polygon[(i + 1) % n];

        if ((y1 > y) != (y2 > y)) && (x < (x2 - x1) * (y - y1) / (y2 - y1) + x1) {
            inside = !inside;
        }
    }

    inside
}

// Check if a point is on the polygon edge (for boundary tiles)
fn point_on_polygon_edge(x: i64, y: i64, polygon: &[(i64, i64)]) -> bool {
    let n = polygon.len();

    for i in 0..n {
        let (x1, y1) = polygon[i];
        let (x2, y2) = polygon[(i + 1) % n];

        // Check if point is on the line segment between (x1,y1) and (x2,y2)
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);

        if x < min_x || x > max_x || y < min_y || y > max_y {
            continue;
        }

        // For horizontal/vertical lines, check if point is collinear
        if x1 == x2 && x == x1 {
            return true;
        }
        if y1 == y2 && y == y1 {
            return true;
        }
    }

    false
}

// Check if a point is red or green (inside/on polygon)
fn is_red_or_green(x: usize, y: usize, polygon: &[(i64, i64)]) -> bool {
    let xi = x as i64;
    let yi = y as i64;
    point_in_polygon(xi, yi, polygon) || point_on_polygon_edge(xi, yi, polygon)
}

// Get the bounding polygon vertices (the red tiles form the outer boundary)
fn get_polygon_bounds(coordinates: &[Coordinate]) -> (usize, usize, usize, usize) {
    let min_x = coordinates.iter().map(|c| c.x).min().unwrap();
    let max_x = coordinates.iter().map(|c| c.x).max().unwrap();
    let min_y = coordinates.iter().map(|c| c.y).min().unwrap();
    let max_y = coordinates.iter().map(|c| c.y).max().unwrap();
    (min_x, max_x, min_y, max_y)
}

fn find_largest_rectangle_in_polygon(coordinates: &[Coordinate]) -> Option<Square> {
    if coordinates.len() < 2 {
        return None;
    }

    // Build the polygon from red tiles
    let polygon: Vec<(i64, i64)> = coordinates
        .iter()
        .map(|c| (c.x as i64, c.y as i64))
        .collect();

    let (poly_min_x, poly_max_x, poly_min_y, poly_max_y) = get_polygon_bounds(coordinates);

    println!("  Polygon bounding box: ({}, {}) to ({}, {})",
             poly_min_x, poly_min_y, poly_max_x, poly_max_y);

    let mut largest_square: Option<Square> = None;
    let mut best_area = 0;

    // Check every pair of RED tile coordinates as potential opposite corners
    for i in 0..coordinates.len() {
        for j in (i + 1)..coordinates.len() {
            let coord1 = coordinates[i];
            let coord2 = coordinates[j];

            // Calculate rectangle bounds
            let min_x = coord1.x.min(coord2.x);
            let max_x = coord1.x.max(coord2.x);
            let min_y = coord1.y.min(coord2.y);
            let max_y = coord1.y.max(coord2.y);

            // Both dimensions must be non-zero to form a rectangle
            if min_x == max_x || min_y == max_y {
                continue;
            }

            // Calculate area
            let area = (max_x - min_x + 1) * (max_y - min_y + 1);

            // Early termination: if this rectangle can't beat the current best, skip it
            if area <= best_area {
                continue;
            }

            // Check if all 4 corners of the rectangle are red or green (inside/on polygon)
            let corners = [
                (min_x, min_y),
                (max_x, min_y),
                (min_x, max_y),
                (max_x, max_y),
            ];

            let corners_valid = corners.iter().all(|&(x, y)| {
                is_red_or_green(x, y, &polygon)
            });

            if !corners_valid {
                continue;
            }

            // Sample points throughout the rectangle to validate it's fully inside
            // Use denser sampling for more accuracy
            let rect_width = max_x - min_x + 1;
            let rect_height = max_y - min_y + 1;

            // Aim for checking ~1000-2000 points regardless of rectangle size
            let sample_size = ((rect_width.max(rect_height) as f64).sqrt() * 10.0).min(100.0) as usize;

            let mut valid = true;

            // Sample points throughout the rectangle
            'check_loop: for sy in 0..=sample_size {
                for sx in 0..=sample_size {
                    let x = min_x + (max_x - min_x) * sx / sample_size;
                    let y = min_y + (max_y - min_y) * sy / sample_size;

                    if !is_red_or_green(x, y, &polygon) {
                        valid = false;
                        break 'check_loop;
                    }
                }
            }

            if !valid {
                continue;
            }

            // Update largest square
            best_area = area;
            largest_square = Some(Square {
                corner1: coord1,
                corner2: coord2,
                area,
            });
        }
    }

    largest_square
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Square {
    corner1: Coordinate,
    corner2: Coordinate,
    area: usize,
}

pub fn run() -> Result<()> {
    // Test with small dataset first
    println!("=== Small dataset (day09tiles1.txt) ===");
    let coordinates1 = parse_input("assets/day09tiles1.txt")?;
    println!("Parsed {} red tile coordinates", coordinates1.len());

    if let Some(square) = find_largest_rectangle(&coordinates1) {
        println!("\nPart 1 - Any tiles: {}", square.area);
    }

    if let Some(square) = find_largest_rectangle_in_polygon(&coordinates1) {
        println!("\nPart 2 - Red/green only:");
        println!("  Corner 1: ({}, {})", square.corner1.x, square.corner1.y);
        println!("  Corner 2: ({}, {})", square.corner2.x, square.corner2.y);
        println!("  Area: {} (expected: 24)", square.area);
    }

    // Large dataset
    println!("\n=== Large dataset (day09tiles2.txt) ===");
    let coordinates2 = parse_input("assets/day09tiles2.txt")?;
    println!("Parsed {} red tile coordinates", coordinates2.len());

    if let Some(square) = find_largest_rectangle(&coordinates2) {
        println!("\nPart 1 - Any tiles: {}", square.area);
    }

    if let Some(square2) = find_largest_rectangle_in_polygon(&coordinates2) {
        println!("\nPart 2 - Red/green only:");
        println!("  Corner 1: ({}, {})", square2.corner1.x, square2.corner1.y);
        println!("  Corner 2: ({}, {})", square2.corner2.x, square2.corner2.y);
        println!("  Area: {}", square2.area);
    } else {
        println!("\nNo valid rectangle found");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1_solution() {
        let coordinates = parse_input("assets/day09tiles1.txt")
            .expect("Failed to load part 1 input");

        let square = find_largest_rectangle(&coordinates)
            .expect("Should find a valid rectangle");

        assert_eq!(square.area, 50, "Part 1 solution should be 50");
    }

    #[test]
    fn test_part1_with_polygon_constraint() {
        let coordinates = parse_input("assets/day09tiles1.txt")
            .expect("Failed to load part 1 input");

        let square = find_largest_rectangle_in_polygon(&coordinates)
            .expect("Should find a valid rectangle");

        assert_eq!(square.area, 24, "Part 1 with polygon constraint should be 24");
    }

    #[test]
    fn test_part2_solution() {
        let coordinates = parse_input("assets/day09tiles2.txt")
            .expect("Failed to load part 2 input");

        let square = find_largest_rectangle(&coordinates)
            .expect("Should find a valid rectangle");

        assert_eq!(square.area, 4740155680, "Part 2 solution should be 4740155680");
    }

    #[test]
    fn test_part2_with_polygon_constraint() {
        let coordinates = parse_input("assets/day09tiles2.txt")
            .expect("Failed to load part 2 input");

        let square = find_largest_rectangle_in_polygon(&coordinates)
            .expect("Should find a valid rectangle");

        assert_eq!(square.area, 1543501936, "Part 2 with polygon constraint should be 1543501936");
    }
}
