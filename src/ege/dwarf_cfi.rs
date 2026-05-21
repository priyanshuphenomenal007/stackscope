use object::{Object, ObjectSection};

use gimli::UnwindSection;

use crate::ege::elf_parser::EgeError;
use crate::types::ExecutionGraph;

/// Extract stack frame geometry from DWARF CFI sections.
pub fn extract_frame_sizes(
    file: &object::File<'_>,
    graph: &mut ExecutionGraph,
) -> Result<(), EgeError> {
    // 1. Locate the .debug_frame section
    let debug_frame_section = match file.section_by_name(".debug_frame") {
        Some(section) => section,
        None => return Ok(()), // No debug frame found, fallback required later
    };

    let section_data = debug_frame_section.uncompressed_data()?;

    // Determine target endianness dynamically
    let endian = if file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    let debug_frame = gimli::DebugFrame::new(&section_data, endian);
    let bases = gimli::BaseAddresses::default();
    let mut ctx = gimli::UnwindContext::new();

    let mut entries = debug_frame.entries(&bases);

    // 2. Iterate over all CFI entries (CIEs and FDEs)
    while let Some(entry) = entries.next()? {
        if let gimli::CieOrFde::Fde(partial_fde) = entry {
            let fde =
                partial_fde.parse(|_, bases, offset| debug_frame.cie_from_offset(bases, offset))?;

            // 1. Truncate 64-bit Gimli artifacts down to 32-bit ARM space.
            // 2. Mask the Thumb mode bit (LSB).
            let initial_address = (fde.initial_address() & 0xFFFFFFFF) & !1;

            let mut max_frame_size: i64 = 0;

            // 3. Evaluate CFA rules across all unwind rows
            let mut table = fde.rows(&debug_frame, &bases, &mut ctx)?;

            while let Some(row) = table.next_row()? {
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

            // 4. Apply the discovered geometry using normalized address
            if let Some(&idx) = graph.address_map.get(&initial_address) {
                if let Some(node) = graph.graph.node_weight_mut(idx) {
                    node.physical_frame_size_bytes = physical_frame_size;
                }
            }
        }
    }

    Ok(())
}
