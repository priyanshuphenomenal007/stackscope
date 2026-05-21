use crate::types::{CallType, ExecutionGraph, WcfgEdge};

use capstone::prelude::*;

use object::{Architecture, Object, ObjectSection};

pub fn build_cfg(file: &object::File<'_>, graph: &mut ExecutionGraph) -> Result<(), String> {
    // Explicitly validate architecture support
    match file.architecture() {
        Architecture::Arm => {}
        other => {
            return Err(format!("Unsupported architecture: {:?}", other));
        }
    }

    // Initialize Capstone for ARM Thumb mode
    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .build()
        .map_err(|e| format!("Capstone init failed: {}", e))?;

    // Extract .text section
    let text_section = file
        .section_by_name(".text")
        .ok_or("No .text section found")?;

    let text_data = text_section
        .uncompressed_data()
        .map_err(|e| e.to_string())?;

    let text_addr = text_section.address();

    // Clone node metadata to avoid borrow conflicts
    let nodes: Vec<_> = graph
        .graph
        .node_indices()
        .map(|idx| {
            let node = &graph.graph[idx];

            (idx, node.physical_address, node.symbol_size_bytes)
        })
        .collect();

    // Disassemble symbol regions and extract call edges
    for (source_idx, addr, symbol_size) in nodes {
        // Avoid unsigned underflow if symbol address is below .text base
        let relative = match addr.checked_sub(text_addr) {
            Some(v) => v,
            None => {
                continue;
            }
        };

        let offset = relative as usize;

        // Skip invalid ranges
        if offset >= text_data.len() || symbol_size == 0 {
            continue;
        }

        // Constrain disassembly exactly to symbol bounds
        let chunk_size = usize::min(symbol_size, text_data.len() - offset);

        let chunk = &text_data[offset..offset + chunk_size];

        if let Ok(insns) = cs.disasm_all(chunk, addr) {
            for insn in insns.as_ref() {
                let mnemonic = insn.mnemonic().unwrap_or("");

                if mnemonic == "bl" || mnemonic == "blx" {
                    if let Some(op_str) = insn.op_str() {
                        // Example operand:
                        // "#0x8028"

                        let clean_op = op_str.trim_start_matches('#').trim_start_matches("0x");

                        if let Ok(target_addr) = u64::from_str_radix(clean_op, 16) {
                            // Normalize Thumb bit
                            let target_normalized = target_addr & !1;

                            // Wire edge if symbol exists
                            if let Some(&target_idx) = graph.address_map.get(&target_normalized) {
                                let edge = WcfgEdge {
                                    call_type: CallType::Direct,
                                };

                                graph.graph.add_edge(source_idx, target_idx, edge);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
