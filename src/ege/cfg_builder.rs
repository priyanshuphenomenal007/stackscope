use crate::types::{CallType, ExecutionGraph, WcfgEdge};
use capstone::prelude::*;
use object::{Object, ObjectSection};

pub fn build_cfg(file: &object::File<'_>, graph: &mut ExecutionGraph) -> Result<(), String> {
    // 1. Initialize Capstone for ARM Thumb mode
    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .build()
        .map_err(|e| format!("Capstone init failed: {}", e))?;

    // 2. Extract the physical .text section
    let text_section = file
        .section_by_name(".text")
        .ok_or("No .text section found")?;
    let text_data = text_section
        .uncompressed_data()
        .map_err(|e| e.to_string())?;
    let text_addr = text_section.address();

    // Clone node indices, addresses, and sizes to avoid borrow checker conflicts
    let nodes: Vec<_> = graph
        .graph
        .node_indices()
        .map(|idx| {
            let node = &graph.graph[idx];
            (idx, node.physical_address, node.symbol_size_bytes)
        })
        .collect();

    // 3. Disassemble and look for Call Instructions (BL, BLX)
    for (source_idx, addr, symbol_size) in nodes {
        let offset = (addr - text_addr) as usize;

        // Skip if out of bounds or if we have no size data
        if offset >= text_data.len() || symbol_size == 0 {
            continue;
        }

        // BOUNDARY FIX: Constrain disassembly chunk exactly to the symbol size
        let chunk_size = usize::min(symbol_size, text_data.len() - offset);
        let chunk = &text_data[offset..offset + chunk_size];

        if let Ok(insns) = cs.disasm_all(chunk, addr) {
            for insn in insns.as_ref() {
                let mnemonic = insn.mnemonic().unwrap_or("");

                if mnemonic == "bl" || mnemonic == "blx" {
                    if let Some(op_str) = insn.op_str() {
                        // Extract target address from operand string (e.g., "#0x8028")
                        let clean_op = op_str.trim_start_matches('#').trim_start_matches("0x");

                        if let Ok(target_addr) = u64::from_str_radix(clean_op, 16) {
                            // Normalize the target Thumb bit
                            let target_normalized = target_addr & !1;

                            // If the target exists in our graph, wire the edge!
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
