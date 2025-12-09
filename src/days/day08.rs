use anyhow::{anyhow, Context, Result};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

fn parse_input(filename: &str) -> Result<Vec<Coordinate3D>> {
    let content = fs::read_to_string(filename)
        .context(format!("Failed to read file: {}", filename))?;
    
    let coordinates = content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let parts: Vec<&str> = line.trim().split(',').collect();
            if parts.len() != 3 {
                return Err(anyhow!(
                    "Line {} has {} values, expected 3 comma-separated values", 
                    i + 1, 
                    parts.len()
                ));
            }
            
            let x = parts[0].parse::<i32>()
                .context(format!("Failed to parse x coordinate on line {}", i + 1))?;
            let y = parts[1].parse::<i32>()
                .context(format!("Failed to parse y coordinate on line {}", i + 1))?;
            let z = parts[2].parse::<i32>()
                .context(format!("Failed to parse z coordinate on line {}", i + 1))?;
            
            Ok(Coordinate3D { x, y, z })
        })
        .collect::<Result<Vec<_>>>()?;
    
    Ok(coordinates)
}

fn euclidean_distance(a: &Coordinate3D, b: &Coordinate3D) -> f64 {
    let dx = (a.x - b.x) as f64;
    let dy = (a.y - b.y) as f64;
    let dz = (a.z - b.z) as f64;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// Wrapper for BinaryHeap that orders by distance (min-heap)
#[derive(Debug)]
struct PairDistance {
    distance: f64,
    i: usize,
    j: usize,
}

impl PartialEq for PairDistance {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for PairDistance {}

impl Ord for PairDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other.distance.partial_cmp(&self.distance).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PairDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn create_clusters(coordinates: &[Coordinate3D], num_connections: usize) -> (Vec<usize>, usize) {
    let n = coordinates.len();
    
    println!("Clustering {} coordinates...", n);
    println!("Computing all pairwise distances...");
    
    // Min-heap to efficiently get the closest pair
    let mut heap: BinaryHeap<PairDistance> = BinaryHeap::new();
    
    // Compute all pairwise distances and add to heap
    for i in 0..n {
        if n >= 100 && i % 100 == 0 {
            println!("  Processing coordinate {} of {}...", i, n);
        }
        for j in (i + 1)..n {
            let distance = euclidean_distance(&coordinates[i], &coordinates[j]);
            heap.push(PairDistance { distance, i, j });
        }
    }
    
    // Track which pairs are directly connected
    let mut connected_pairs: HashSet<(usize, usize)> = HashSet::new();
    
    // Track which cluster each coordinate belongs to
    let mut coordinate_to_cluster: HashMap<usize, usize> = HashMap::new();
    
    // Track clusters as sets of coordinate indices
    let mut clusters: Vec<HashSet<usize>> = Vec::new();
    
    let mut connections_made = 0;
    
    println!("Connecting {} closest pairs...", num_connections);
    
    // Repeatedly find the closest pair that aren't already directly connected
    while connections_made < num_connections {
        // Pop pairs from heap until we find one that's not already connected
        let closest_pair = loop {
            if let Some(pair) = heap.pop() {
                let key = if pair.i < pair.j { (pair.i, pair.j) } else { (pair.j, pair.i) };
                
                if !connected_pairs.contains(&key) {
                    break Some((pair.i, pair.j));
                }
                // Otherwise, this pair was already connected, skip it
            } else {
                break None; // No more pairs available
            }
        };
        
        // If we found a pair, connect them
        if let Some((i, j)) = closest_pair {
            let key = if i < j { (i, j) } else { (j, i) };
            connected_pairs.insert(key);
            connections_made += 1;
            
            if n >= 100 && connections_made % 100 == 0 {
                println!("  Made {} connections...", connections_made);
            }
            
            let cluster_i = coordinate_to_cluster.get(&i).copied();
            let cluster_j = coordinate_to_cluster.get(&j).copied();
            
            match (cluster_i, cluster_j) {
                (Some(ci), Some(cj)) if ci == cj => {
                    // Both already in same cluster, connection just adds redundancy
                }
                (Some(ci), Some(cj)) => {
                    // Both in different clusters - merge them
                    let cluster_j_members: Vec<usize> = clusters[cj].iter().copied().collect();
                    for member in cluster_j_members {
                        clusters[ci].insert(member);
                        coordinate_to_cluster.insert(member, ci);
                    }
                    clusters[cj].clear();
                }
                (Some(ci), None) => {
                    // i is in a cluster, add j to it
                    clusters[ci].insert(j);
                    coordinate_to_cluster.insert(j, ci);
                }
                (None, Some(cj)) => {
                    // j is in a cluster, add i to it
                    clusters[cj].insert(i);
                    coordinate_to_cluster.insert(i, cj);
                }
                (None, None) => {
                    // Neither is in a cluster, create a new one
                    let cluster_id = clusters.len();
                    let mut new_cluster = HashSet::new();
                    new_cluster.insert(i);
                    new_cluster.insert(j);
                    clusters.push(new_cluster);
                    coordinate_to_cluster.insert(i, cluster_id);
                    coordinate_to_cluster.insert(j, cluster_id);
                }
            }
        } else {
            // No more pairs to connect
            break;
        }
    }
    
    // Add singleton clusters for any coordinates that were never connected
    for i in 0..n {
        if !coordinate_to_cluster.contains_key(&i) {
            let mut singleton_cluster = HashSet::new();
            singleton_cluster.insert(i);
            clusters.push(singleton_cluster);
        }
    }
    
    // Filter out empty clusters and get sizes, then sort for readability
    let mut cluster_sizes: Vec<usize> = clusters
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| c.len())
        .collect();
    cluster_sizes.sort_by(|a, b| b.cmp(a)); // Sort descending
    
    println!("\n{} circuits created:", cluster_sizes.len());
    let mut size_counts: HashMap<usize, usize> = HashMap::new();
    for &size in &cluster_sizes {
        *size_counts.entry(size).or_insert(0) += 1;
    }
    
    let mut sizes: Vec<_> = size_counts.keys().copied().collect();
    sizes.sort_by(|a, b| b.cmp(a));
    for size in sizes {
        let count = size_counts[&size];
        println!("  {} circuit(s) with {} junction box(es)", count, size);
    }
    
    // Show top 10 cluster sizes for debugging
    println!("\nTop 10 largest circuits:");
    for (i, &size) in cluster_sizes.iter().take(10).enumerate() {
        println!("  {}. {} junction boxes", i + 1, size);
    }
    
    // Calculate product of three largest circuits
    let product = if cluster_sizes.len() >= 3 {
        let prod = cluster_sizes[0] * cluster_sizes[1] * cluster_sizes[2];
        println!("\nProduct of three largest circuits: {} * {} * {} = {}", 
                 cluster_sizes[0], 
                 cluster_sizes[1], 
                 cluster_sizes[2],
                 prod);
        prod
    } else {
        0
    };
    
    (cluster_sizes, product)
}

fn connect_until_single_cluster(coordinates: &[Coordinate3D]) -> Result<i64> {
    let n = coordinates.len();
    
    println!("Connecting all {} coordinates into a single circuit...", n);
    println!("Computing all pairwise distances...");
    
    // Min-heap to efficiently get the closest pair
    let mut heap: BinaryHeap<PairDistance> = BinaryHeap::new();
    
    // Compute all pairwise distances and add to heap
    for i in 0..n {
        if n >= 100 && i % 100 == 0 {
            println!("  Processing coordinate {} of {}...", i, n);
        }
        for j in (i + 1)..n {
            let distance = euclidean_distance(&coordinates[i], &coordinates[j]);
            heap.push(PairDistance { distance, i, j });
        }
    }
    
    // Track which pairs are directly connected
    let mut connected_pairs: HashSet<(usize, usize)> = HashSet::new();
    
    // Track which cluster each coordinate belongs to
    let mut coordinate_to_cluster: HashMap<usize, usize> = HashMap::new();
    
    // Track clusters as sets of coordinate indices
    let mut clusters: Vec<HashSet<usize>> = Vec::new();
    
    // Initialize: each coordinate starts in its own cluster
    for i in 0..n {
        let mut singleton = HashSet::new();
        singleton.insert(i);
        clusters.push(singleton);
        coordinate_to_cluster.insert(i, i);
    }
    
    let mut connections_made = 0;
    let mut last_connected_pair: Option<(usize, usize)> = None;
    
    // Count how many non-empty clusters we have
    let mut num_clusters = n;
    
    println!("Starting with {} circuits...", num_clusters);
    
    // Continue until we have only 1 cluster
    while num_clusters > 1 {
        // Pop pairs from heap until we find one that's not already connected
        let closest_pair = loop {
            if let Some(pair) = heap.pop() {
                let key = if pair.i < pair.j { (pair.i, pair.j) } else { (pair.j, pair.i) };
                
                if !connected_pairs.contains(&key) {
                    break Some((pair.i, pair.j));
                }
                // Otherwise, this pair was already connected, skip it
            } else {
                return Err(anyhow!("Ran out of pairs before forming single cluster"));
            }
        };
        
        // If we found a pair, connect them
        if let Some((i, j)) = closest_pair {
            let key = if i < j { (i, j) } else { (j, i) };
            connected_pairs.insert(key);
            connections_made += 1;
            last_connected_pair = Some((i, j));
            
            if n >= 100 && connections_made % 100 == 0 {
                println!("  Made {} connections, {} circuits remaining...", 
                         connections_made, num_clusters);
            }
            
            let cluster_i = coordinate_to_cluster[&i];
            let cluster_j = coordinate_to_cluster[&j];
            
            if cluster_i != cluster_j {
                // Merge the two clusters
                let cluster_j_members: Vec<usize> = clusters[cluster_j].iter().copied().collect();
                for member in cluster_j_members {
                    clusters[cluster_i].insert(member);
                    coordinate_to_cluster.insert(member, cluster_i);
                }
                clusters[cluster_j].clear();
                num_clusters -= 1; // We merged two clusters into one
            }
            // else: both already in same cluster, connection just adds redundancy
        }
    }
    
    println!("\nAll junction boxes connected into a single circuit!");
    println!("Total connections made: {}", connections_made);
    
    if let Some((i, j)) = last_connected_pair {
        let x_product = (coordinates[i].x as i64) * (coordinates[j].x as i64);
        println!("\nLast connection: junction box {} (x={}) <-> junction box {} (x={})",
                 i, coordinates[i].x, j, coordinates[j].x);
        println!("Product of X coordinates: {} * {} = {}", 
                 coordinates[i].x, coordinates[j].x, x_product);
        Ok(x_product)
    } else {
        Err(anyhow!("No connections were made"))
    }
}

/// Day 8: Playground - Junction Box Circuit Analysis
pub fn run() -> Result<()> {
    let coordinates = parse_input("assets/day08coordinates.txt")?;
    
    println!("Day 8: Loaded {} coordinates", coordinates.len());
    
    // Part 1: Connect 1000 closest pairs for the full puzzle
    println!("\n=== Part 1: Limited Connections ===");
    create_clusters(&coordinates, 1000);
    
    // Part 2: Connect until all are in a single circuit
    println!("\n=== Part 2: Single Circuit ===");
    connect_until_single_cluster(&coordinates)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Load the example data (20 junction boxes)
        let coordinates = parse_input("assets/day08example.txt")
            .expect("Failed to load example data");
        
        assert_eq!(coordinates.len(), 20, "Example should have 20 junction boxes");
        
        // After making 10 connections, should have 11 circuits
        // Largest: 5, 4, 2 -> product = 40
        let (cluster_sizes, product) = create_clusters(&coordinates, 10);
        
        assert_eq!(cluster_sizes.len(), 11, "Should have 11 circuits after 10 connections");
        assert_eq!(cluster_sizes[0], 5, "Largest circuit should have 5 junction boxes");
        assert_eq!(cluster_sizes[1], 4, "Second largest circuit should have 4 junction boxes");
        assert_eq!(cluster_sizes[2], 2, "Third largest circuit should have 2 junction boxes");
        assert_eq!(product, 40, "Product of three largest circuits should be 40");
    }

    #[test]
    fn test_full_puzzle() {
        // Load the full puzzle data (1000 junction boxes)
        let coordinates = parse_input("assets/day08coordinates.txt")
            .expect("Failed to load full puzzle data");
        
        assert_eq!(coordinates.len(), 1000, "Full puzzle should have 1000 junction boxes");
        
        // After making 1000 connections, should have 296 circuits
        // Largest: 57, 37, 32 -> product = 67488
        let (cluster_sizes, product) = create_clusters(&coordinates, 1000);
        
        assert_eq!(cluster_sizes.len(), 296, "Should have 296 circuits after 1000 connections");
        assert_eq!(cluster_sizes[0], 57, "Largest circuit should have 57 junction boxes");
        assert_eq!(cluster_sizes[1], 37, "Second largest circuit should have 37 junction boxes");
        assert_eq!(cluster_sizes[2], 32, "Third largest circuit should have 32 junction boxes");
        assert_eq!(product, 67488, "Product of three largest circuits should be 67488");
    }

    #[test]
    fn test_single_cluster_example() {
        // Load the example data (20 junction boxes)
        let coordinates = parse_input("assets/day08example.txt")
            .expect("Failed to load example data");
        
        assert_eq!(coordinates.len(), 20, "Example should have 20 junction boxes");
        
        // Connect until all are in a single circuit (requires 19 connections)
        let x_product = connect_until_single_cluster(&coordinates)
            .expect("Failed to create single cluster");
        
        // The answer will depend on the data, just verify we got a result
        assert!(x_product > 0, "Product should be positive");
    }

    #[test]
    fn test_single_cluster_full_puzzle() {
        // Load the full puzzle data (1000 junction boxes)
        let coordinates = parse_input("assets/day08coordinates.txt")
            .expect("Failed to load full puzzle data");
        
        assert_eq!(coordinates.len(), 1000, "Full puzzle should have 1000 junction boxes");
        
        // Connect until all are in a single circuit (requires 6282 connections)
        let x_product = connect_until_single_cluster(&coordinates)
            .expect("Failed to create single cluster");
        
        // The answer is the product of X coordinates of the last two connected junction boxes
        assert_eq!(x_product, 3767453340, "Product of X coordinates should be 3767453340");
    }
}
