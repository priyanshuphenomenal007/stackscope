use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Physical function representation extracted from ELF symbols.
#[derive(Debug, Clone)]
pub struct WcfgNode {
    /// Physical address in executable memory.
    pub physical_address: u64,

    /// ELF symbol name.
    pub symbol_name: String,

    /// Stack frame allocation in bytes.
    pub physical_frame_size_bytes: usize,

    /// Exact size of the function symbol in bytes.
    pub symbol_size_bytes: usize,
}

/// Edge classification for control-flow transitions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CallType {
    Direct,
    Indirect,
    HardwareInterrupt(u8),
}

/// Weighted CFG edge.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WcfgEdge {
    pub call_type: CallType,
}

/// Weighted Control-Flow Graph.
#[derive(Debug, Clone)]
pub struct ExecutionGraph {
    pub graph: DiGraph<WcfgNode, WcfgEdge>,

    /// Fast lookup from physical address → graph node.
    pub address_map: HashMap<u64, NodeIndex>,
}

impl ExecutionGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            address_map: HashMap::new(),
        }
    }
}
