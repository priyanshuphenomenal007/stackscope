use object::{Object, ObjectSection};

use gimli::UnwindSection;

use crate::ege::elf_parser::EgeError;
use crate::types::ExecutionGraph;

/// Extract stack frame geometry from DWARF CFI sections.
///
/// Real firmware often contains:
/// - partial unwind metadata
/// - linker-modified DWARF
/// - compressed sections
/// - malformed FDE entries
///
/// This layer should NEVER abort ingestion completely.
/// Failures degrade gracefully into missing frame data.
pub fn extract_frame_sizes(
    file: &object::File<'_>,
    graph: &mut ExecutionGraph,
) -> Result<(), EgeError> {
    eprintln!("[DWARF] Starting frame extraction");

    // Locate .debug_frame section
    let debug_frame_section = match file.section_by_name(".debug_frame") {
        Some(section) => {
            eprintln!("[DWARF] Found .debug_frame section");
            section
        }

        None => {
            eprintln!("[DWARF] No .debug_frame section found");

            return Ok(());
        }
    };

    let section_data = match debug_frame_section.uncompressed_data() {
        Ok(data) => {
            eprintln!("[DWARF] Loaded .debug_frame data ({} bytes)", data.len());

            data
        }

        Err(err) => {
            eprintln!("[DWARF] Failed to load .debug_frame: {}", err);

            // Graceful degradation
            return Ok(());
        }
    };

    // Determine endianness dynamically
    let endian = if file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    let debug_frame = gimli::DebugFrame::new(&section_data, endian);

    let bases = gimli::BaseAddresses::default();

    let mut ctx = gimli::UnwindContext::new();

    let mut entries = debug_frame.entries(&bases);

    eprintln!("[DWARF] Beginning CFI traversal");

    loop {
        let next_entry = match entries.next() {
            Ok(v) => v,

            Err(err) => {
                eprintln!("[DWARF] Entry traversal failed: {}", err);

                // Graceful degradation
                break;
            }
        };

        let entry = match next_entry {
            Some(e) => e,

            None => {
                eprintln!("[DWARF] Completed CFI traversal");

                break;
            }
        };

        match entry {
            gimli::CieOrFde::Cie(_) => {
                // Ignore CIE entries directly
            }

            gimli::CieOrFde::Fde(partial_fde) => {
                let fde = match partial_fde
                    .parse(|_, bases, offset| debug_frame.cie_from_offset(bases, offset))
                {
                    Ok(fde) => fde,

                    Err(err) => {
                        eprintln!("[DWARF] Failed to parse FDE: {}", err);

                        continue;
                    }
                };

                // Normalize ARM Thumb addresses
                let initial_address = (fde.initial_address() & 0xFFFFFFFF) & !1;

                let mut max_frame_size: i64 = 0;

                let mut table = match fde.rows(&debug_frame, &bases, &mut ctx) {
                    Ok(t) => t,

                    Err(err) => {
                        eprintln!("[DWARF] Failed to load unwind rows: {}", err);

                        continue;
                    }
                };

                loop {
                    let next_row = match table.next_row() {
                        Ok(v) => v,

                        Err(err) => {
                            eprintln!("[DWARF] Row traversal failed: {}", err);

                            break;
                        }
                    };

                    let row = match next_row {
                        Some(r) => r,

                        None => break,
                    };

                    match row.cfa() {
                        gimli::CfaRule::RegisterAndOffset { offset, .. } => {
                            let offset_value = *offset;

                            if offset_value > max_frame_size {
                                max_frame_size = offset_value;
                            }
                        }

                        gimli::CfaRule::Expression(_) => {
                            // Ignore expression-based CFA rules for now.
                        }
                    }
                }

                let physical_frame_size = max_frame_size.max(0) as usize;

                if let Some(&idx) = graph.address_map.get(&initial_address) {
                    if let Some(node) = graph.graph.node_weight_mut(idx) {
                        node.physical_frame_size_bytes = physical_frame_size;
                    }
                }
            }
        }
    }

    eprintln!("[DWARF] Frame extraction completed");

    Ok(())
}
