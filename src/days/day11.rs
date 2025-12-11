use anyhow::{anyhow, Context, Result};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::rc::Rc;

/// Node in the graph
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(id: String) -> Self {
        Node {
            id,
            children: Vec::new(),
        }
    }
}

fn parse_input(filename: &str, root_id: &str) -> Result<Rc<RefCell<Node>>> {
    let content = fs::read_to_string(filename)
        .context(format!("Failed to read file: {}", filename))?;

    // First pass: create all nodes
    let mut nodes: HashMap<String, Rc<RefCell<Node>>> = HashMap::new();
    let mut edges: Vec<(String, Vec<String>)> = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "Line {} has invalid format, expected 'id: child1 child2 ...'",
                i + 1
            ));
        }

        let node_id = parts[0].trim().to_string();
        let children_ids: Vec<String> = parts[1]
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Create node if it doesn't exist
        if !nodes.contains_key(&node_id) {
            nodes.insert(node_id.clone(), Rc::new(RefCell::new(Node::new(node_id.clone()))));
        }

        // Create child nodes if they don't exist
        for child_id in &children_ids {
            if !nodes.contains_key(child_id) {
                nodes.insert(
                    child_id.clone(),
                    Rc::new(RefCell::new(Node::new(child_id.clone()))),
                );
            }
        }

        edges.push((node_id, children_ids));
    }

    // Second pass: connect nodes
    for (parent_id, children_ids) in edges {
        let parent = nodes
            .get(&parent_id)
            .ok_or_else(|| anyhow!("Parent node '{}' not found", parent_id))?;

        for child_id in children_ids {
            let child = nodes
                .get(&child_id)
                .ok_or_else(|| anyhow!("Child node '{}' not found", child_id))?;
            parent.borrow_mut().children.push(Rc::clone(child));
        }
    }

    // Find and return the specified root node
    nodes
        .get(root_id)
        .cloned()
        .ok_or_else(|| anyhow!("Root node '{}' not found in input", root_id))
}

/// Count the number of unique paths from a given node to 'out' nodes
fn count_paths_to_out(node: &Rc<RefCell<Node>>) -> usize {
    let node_ref = node.borrow();
    
    // Base case: if this is an 'out' node, we found one path
    if node_ref.id == "out" {
        return 1;
    }
    
    // Recursive case: sum up paths from all children
    node_ref
        .children
        .iter()
        .map(|child| count_paths_to_out(child))
        .sum()
}

/// Count paths from current node to 'out', but only paths that include all required nodes
/// Uses memoization to avoid recomputing the same subproblems
fn count_paths_with_required_memo(
    node: &Rc<RefCell<Node>>,
    visited_required: HashSet<String>,
    visited_in_path: HashSet<String>,
    required_nodes: &HashSet<String>,
    memo: &mut HashMap<(String, Vec<String>), usize>,
) -> usize {
    let node_ref = node.borrow();
    let node_id = node_ref.id.clone();
    
    // Cycle detection: if we've already visited this node in the current path, return 0
    if visited_in_path.contains(&node_id) {
        return 0;
    }
    
    // Create a cache key: (node_id, sorted list of required nodes we've visited)
    let mut visited_req_sorted: Vec<String> = visited_required.iter().cloned().collect();
    visited_req_sorted.sort();
    let cache_key = (node_id.clone(), visited_req_sorted);
    
    // Check memo cache
    if let Some(&cached_result) = memo.get(&cache_key) {
        return cached_result;
    }
    
    // Mark this node as visited in the current path
    let mut new_visited_in_path = visited_in_path.clone();
    new_visited_in_path.insert(node_id.clone());
    
    // Track if this node is one of the required ones
    let mut new_visited_required = visited_required.clone();
    if required_nodes.contains(&node_id) {
        new_visited_required.insert(node_id.clone());
    }
    
    // Base case: if this is an 'out' node
    let result = if node_id == "out" {
        // Only count this path if we've visited all required nodes
        if new_visited_required.len() == required_nodes.len() {
            1
        } else {
            0
        }
    } else {
        // Recursive case: sum up valid paths from all children
        node_ref
            .children
            .iter()
            .map(|child| {
                count_paths_with_required_memo(
                    child,
                    new_visited_required.clone(),
                    new_visited_in_path.clone(),
                    required_nodes,
                    memo,
                )
            })
            .sum()
    };
    
    // Cache the result
    memo.insert(cache_key, result);
    result
}

/// Count the number of unique paths from 'svr' to 'out' that include both 'dac' and 'fft'
fn count_paths_from_svr(root: &Rc<RefCell<Node>>) -> usize {
    let mut required_nodes = HashSet::new();
    required_nodes.insert("dac".to_string());
    required_nodes.insert("fft".to_string());
    
    let mut memo = HashMap::new();
    count_paths_with_required_memo(
        root,
        HashSet::new(),
        HashSet::new(),
        &required_nodes,
        &mut memo,
    )
}

/// Day 11: Exercise description
pub fn run() -> Result<()> {
    // Part 1
    println!("Part 1:");
    let root1 = parse_input("assets/day11io1.txt", "you")?;
    let num_paths1 = count_paths_to_out(&root1);
    println!("  Number of unique paths from 'you' to 'out': {}", num_paths1);
    
    // Part 2
    println!("\nPart 2:");
    let root2 = parse_input("assets/day11io2.txt", "you")?;
    let num_paths2 = count_paths_to_out(&root2);
    println!("  Number of unique paths from 'you' to 'out': {}", num_paths2);
    
    // Part 2b - from 'svr' with constraints
    println!("\nPart 2b:");
    let root2b = parse_input("assets/day11io2.txt", "svr")?;
    let num_paths2b = count_paths_from_svr(&root2b);
    println!("  Number of paths from 'svr' to 'out' including both 'dac' and 'fft': {}", num_paths2b);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1_path_count() {
        let root = parse_input("assets/day11io1.txt", "you")
            .expect("Failed to load part 1 input");
        
        let num_paths = count_paths_to_out(&root);
        
        assert_eq!(num_paths, 5, "Part 1 should have 5 unique paths");
    }

    #[test]
    fn test_part2_path_count() {
        let root = parse_input("assets/day11io2.txt", "you")
            .expect("Failed to load part 2 input");
        
        let num_paths = count_paths_to_out(&root);
        
        assert_eq!(num_paths, 701, "Part 2 should have 701 unique paths");
    }

    #[test]
    fn test_part2b_svr_with_constraints() {
        let root = parse_input("assets/day11io2.txt", "svr")
            .expect("Failed to load part 2 input");
        
        let num_paths = count_paths_from_svr(&root);
        
        assert_eq!(
            num_paths, 390108778818526,
            "Part 2b should have 390108778818526 paths from 'svr' to 'out' including both 'dac' and 'fft'"
        );
    }
}
