use crate::types::ExecutionGraph;

use petgraph::algo::tarjan_scc;
use petgraph::visit::EdgeRef;

/// Information about a detected recursion cycle.
#[allow(dead_code)]
#[derive(Debug)]
pub struct CycleInfo {
    /// Physical addresses participating in the cycle.
    pub participating_addresses: Vec<u64>,

    /// Symbol names participating in the cycle.
    pub symbols: Vec<String>,
}

#[allow(dead_code)]
pub struct CycleDetector;

#[allow(dead_code)]
impl CycleDetector {
    /// Detect recursion cycles in the execution graph.
    pub fn detect_cycles(eg: &ExecutionGraph) -> Vec<CycleInfo> {
        let mut found_cycles = Vec::new();

        // Strongly Connected Components
        let sccs = tarjan_scc(&eg.graph);

        for scc in sccs {
            let is_multi_node = scc.len() > 1;

            // Detect self-recursion
            let is_self_loop = if scc.len() == 1 {
                let node_idx = scc[0];

                eg.graph.edges(node_idx).any(|e| e.target() == node_idx)
            } else {
                false
            };

            if is_multi_node || is_self_loop {
                let mut addresses = Vec::new();
                let mut symbols = Vec::new();

                for node_idx in scc {
                    let node = &eg.graph[node_idx];

                    addresses.push(node.physical_address);
                    symbols.push(node.symbol_name.clone());
                }

                found_cycles.push(CycleInfo {
                    participating_addresses: addresses,
                    symbols,
                });
            }
        }

        found_cycles
    }
}
