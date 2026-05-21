use crate::types::ExecutionGraph;

use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use serde::Serialize;

use std::collections::HashSet;

/// Final WCSD analysis result.
#[derive(Debug, Serialize)]
pub struct WcsdResult {
    /// Maximum bounded stack depth discovered.
    pub max_depth_bytes: usize,

    /// Whether recursion caused the path to become unbounded.
    pub is_unbounded: bool,

    /// Critical path that produced the maximum depth.
    pub critical_path: Vec<String>,

    /// Whether the configured stack budget was exceeded.
    pub budget_exceeded: bool,
}

pub struct WcsdEngine;

impl WcsdEngine {
    /// Analyze stack depth starting from an entry node.
    pub fn analyze(eg: &ExecutionGraph, entry_node: NodeIndex, budget: usize) -> WcsdResult {
        let mut max_depth = 0;

        let mut is_unbounded = false;

        let mut critical_path = Vec::new();

        let mut current_path = Vec::new();

        let mut path_set = HashSet::new();

        Self::dfs(
            eg,
            entry_node,
            0,
            &mut current_path,
            &mut path_set,
            &mut max_depth,
            &mut is_unbounded,
            &mut critical_path,
        );

        WcsdResult {
            max_depth_bytes: max_depth,

            is_unbounded,

            critical_path,

            budget_exceeded: is_unbounded || max_depth > budget,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs(
        eg: &ExecutionGraph,
        current: NodeIndex,
        current_depth: usize,
        current_path: &mut Vec<NodeIndex>,
        path_set: &mut HashSet<NodeIndex>,
        max_depth: &mut usize,
        is_unbounded: &mut bool,
        critical_path: &mut Vec<String>,
    ) {
        // Recursion / cycle detection
        if path_set.contains(&current) {
            *is_unbounded = true;
            return;
        }

        let node = &eg.graph[current];

        let new_depth = current_depth + node.physical_frame_size_bytes;

        // Push traversal state
        current_path.push(current);

        path_set.insert(current);

        // Track deepest path
        if new_depth > *max_depth {
            *max_depth = new_depth;

            *critical_path = current_path
                .iter()
                .map(|&idx| eg.graph[idx].symbol_name.clone())
                .collect();
        }

        // Traverse outbound edges
        for edge in eg
            .graph
            .edges_directed(current, petgraph::Direction::Outgoing)
        {
            Self::dfs(
                eg,
                edge.target(),
                new_depth,
                current_path,
                path_set,
                max_depth,
                is_unbounded,
                critical_path,
            );
        }

        // Backtrack
        path_set.remove(&current);

        current_path.pop();
    }
}
