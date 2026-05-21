use crate::types::{CallType, ExecutionGraph, WcfgEdge};

use capstone::prelude::*;
use object::{Object, ObjectSection, SectionKind};

pub fn build_cfg(file: &object::File<'_>, graph: &mut ExecutionGraph) -> Result<(), String> {
    // Initialize Capstone for ARM Thumb mode
    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .build()
        .map_err(|e| format!("Capstone init failed: {}", e))?;

    // Collect ALL executable text sections
    let executable_sections: Vec<_> = file
        .sections()
        .filter(|section| section.kind() == SectionKind::Text)
        .collect();

    if executable_sections.is_empty() {
        return Err("No executable text sections found".to_string());
    }

    // Clone node metadata to avoid borrow conflicts
    let nodes: Vec<_> = graph
        .graph
        .node_indices()
        .map(|idx| {
            let node = &graph.graph[idx];

            (idx, node.physical_address, node.symbol_size_bytes)
        })
        .collect();

    // Build CFG edges
    for (source_idx, addr, symbol_size) in nodes {
        if symbol_size == 0 {
            continue;
        }

        let mut matched_section = None;

        for section in &executable_sections {
            let section_addr = section.address();

            let section_data = match section.uncompressed_data() {
                Ok(data) => data,

                Err(_) => continue,
            };

            let section_end = section_addr + section_data.len() as u64;

            if addr >= section_addr && addr < section_end {
                matched_section = Some((section_addr, section_data));

                break;
            }
        }

        let (section_addr, section_data) = match matched_section {
            Some(v) => v,

            None => continue,
        };

        let relative = match addr.checked_sub(section_addr) {
            Some(v) => v,

            None => continue,
        };

        let offset = relative as usize;

        if offset >= section_data.len() {
            continue;
        }

        let chunk_size = usize::min(symbol_size, section_data.len() - offset);

        let chunk = &section_data[offset..offset + chunk_size];

        if let Ok(insns) = cs.disasm_all(chunk, addr) {
            for insn in insns.as_ref() {
                let mnemonic = insn.mnemonic().unwrap_or("");

                if mnemonic == "bl" || mnemonic == "blx" {
                    if let Some(op_str) = insn.op_str() {
                        let clean_op = op_str.trim_start_matches('#').trim_start_matches("0x");

                        if let Ok(target_addr) = u64::from_str_radix(clean_op, 16) {
                            let target_normalized = target_addr & !1;

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
