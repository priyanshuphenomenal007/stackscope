use std::fs;
use std::path::Path;

use object::{Object, ObjectSymbol, SymbolKind};
use thiserror::Error;

use crate::ege::cfg_builder;
use crate::types::{ExecutionGraph, WcfgNode};

#[derive(Error, Debug)]
pub enum EgeError {
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ELF parsing error: {0}")]
    ObjectParse(#[from] object::Error),

    #[error("DWARF parsing error: {0}")]
    DwarfParse(#[from] gimli::Error),

    #[error("Unsupported architecture: {0}")]
    UnsupportedArchitecture(String),
}

/// Load ELF symbols and DWARF geometry into the execution graph.
pub fn load_elf(file_path: &Path) -> Result<ExecutionGraph, EgeError> {
    let binary = fs::read(file_path)?;
    let file = object::File::parse(&*binary)?;
    let mut graph = ExecutionGraph::new();

    // 1. Establish structural topology
    for symbol in file.symbols() {
        if symbol.kind() != SymbolKind::Text {
            continue;
        }

        let raw_address = symbol.address();
        if raw_address == 0 {
            continue;
        }

        // NORMALIZATION: Mask the Thumb bit (LSB)
        let normalized_address = raw_address & !1;
        let name = symbol.name().unwrap_or("<stripped>").to_string();

        // Extract the exact symbol size
        let symbol_size = symbol.size() as usize;

        let node = WcfgNode {
            physical_address: normalized_address,
            symbol_name: name,
            physical_frame_size_bytes: 0,
            symbol_size_bytes: symbol_size,
        };

        let node_index = graph.graph.add_node(node);

        // Map the normalized address
        graph.address_map.insert(normalized_address, node_index);
    }

    // 2. Project physical stack geometry onto the topology
    crate::ege::dwarf_cfi::extract_frame_sizes(&file, &mut graph)?;

    // 3. Extract Control Flow Topology via disassembly
    cfg_builder::build_cfg(&file, &mut graph).map_err(|e| EgeError::UnsupportedArchitecture(e))?;

    Ok(graph)
}
