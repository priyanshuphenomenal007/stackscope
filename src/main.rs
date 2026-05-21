mod ege;
mod mga;
mod sdt;
mod types;

use std::path::Path;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};

use ege::elf_parser::load_elf;
use sdt::abstract_interpreter::WcsdEngine;
use sdt::diff_engine::WcsdDiffResult;

const SCHEMA_VERSION: &str = "1.0";

#[derive(Parser)]
#[command(name = "stackscope")]
#[command(about = "Machine-space stack geometry and divergence analysis")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a single ELF binary
    Analyze {
        #[arg(short, long)]
        elf: String,

        #[arg(long, default_value = "main")]
        entry: String,

        #[arg(short, long, default_value_t = 4096)]
        budget: usize,

        #[arg(short, long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },

    /// Compare two ELF binaries
    Diff {
        #[arg(long)]
        baseline: String,

        #[arg(long)]
        candidate: String,

        #[arg(long, default_value = "main")]
        entry: String,

        #[arg(short, long, value_enum, default_value_t = Format::Json)]
        format: Format,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    Text,
    Json,
    Sarif,
}

/// Stable process exit codes for CI systems.
#[allow(dead_code)]
#[repr(i32)]
enum ExitCode {
    Success = 0,
    PolicyViolation = 1,
    InvalidConfig = 2,
    ElfParseFailure = 3,
    EntryNotFound = 4,
    UnsupportedArchitecture = 5,
    InternalAnalysisFailure = 6,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze {
            elf,
            entry,
            budget,
            format,
        } => {
            run_analyze(elf, entry, *budget, *format);
        }

        Commands::Diff {
            baseline,
            candidate,
            entry,
            format,
        } => {
            run_diff(baseline, candidate, entry, *format);
        }
    }
}

fn run_analyze(elf: &str, entry: &str, budget: usize, format: Format) {
    let elf_path = Path::new(elf);

    let graph = match load_elf(elf_path) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("ELF ingestion failed: {}", err);
            process::exit(ExitCode::ElfParseFailure as i32);
        }
    };

    let entry_node = graph
        .graph
        .node_indices()
        .find(|&idx| graph.graph[idx].symbol_name == entry);

    let entry_idx = match entry_node {
        Some(idx) => idx,
        None => {
            eprintln!("Entry point '{}' not found.", entry);
            process::exit(ExitCode::EntryNotFound as i32);
        }
    };

    let result = WcsdEngine::analyze(&graph, entry_idx, budget);

    match format {
        Format::Json => {
            let payload = serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "analysis_type": "analyze",
                "result": result
            });

            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        }

        Format::Sarif => {
            let mut results = vec![];

            if result.budget_exceeded {
                let message = if result.is_unbounded {
                    "Unbounded stack growth (recursive cycle) detected.".to_string()
                } else {
                    format!(
                        "Stack budget exceeded. Max depth: {} bytes (Budget: {} bytes)",
                        result.max_depth_bytes, budget
                    )
                };

                // Map the critical path into SARIF locations
                let locations: Vec<_> = result
                    .critical_path
                    .iter()
                    .map(|sym| {
                        serde_json::json!({
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": elf
                                },
                                "region": {
                                    "snippet": {
                                        "text": sym
                                    }
                                }
                            }
                        })
                    })
                    .collect();

                results.push(serde_json::json!({
                    "ruleId": "STACK-001",
                    "level": "error",
                    "message": {
                        "text": message
                    },
                    "locations": locations
                }));
            }

            let sarif_payload = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
                "runs": [
                    {
                        "tool": {
                            "driver": {
                                "name": "StackScope",
                                "informationUri": "https://github.com/stackscope/stackscope",
                                "rules": [
                                    {
                                        "id": "STACK-001",
                                        "name": "StackBudgetExceeded",
                                        "shortDescription": {
                                            "text": "Firmware stack budget exceeded."
                                        },
                                        "helpUri": "https://github.com/stackscope/stackscope/wiki/STACK-001"
                                    }
                                ]
                            }
                        },
                        "results": results
                    }
                ]
            });

            println!("{}", serde_json::to_string_pretty(&sarif_payload).unwrap());
        }

        Format::Text => {
            println!("=== WCSD Analysis ===");

            if result.is_unbounded {
                println!("Result: UNBOUNDED");
            } else {
                println!("Maximum Stack Depth: {} bytes", result.max_depth_bytes);
            }

            println!("Critical Path:");

            for symbol in &result.critical_path {
                println!("    -> {}", symbol);
            }
        }
    }

    if result.budget_exceeded {
        process::exit(ExitCode::PolicyViolation as i32);
    }

    process::exit(ExitCode::Success as i32);
}

fn run_diff(baseline: &str, candidate: &str, entry: &str, format: Format) {
    let baseline_graph = match load_elf(Path::new(baseline)) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("Baseline ELF failed: {}", err);
            process::exit(ExitCode::ElfParseFailure as i32);
        }
    };

    let candidate_graph = match load_elf(Path::new(candidate)) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("Candidate ELF failed: {}", err);
            process::exit(ExitCode::ElfParseFailure as i32);
        }
    };

    let baseline_entry = baseline_graph
        .graph
        .node_indices()
        .find(|&idx| baseline_graph.graph[idx].symbol_name == entry);

    let candidate_entry = candidate_graph
        .graph
        .node_indices()
        .find(|&idx| candidate_graph.graph[idx].symbol_name == entry);

    let baseline_entry = match baseline_entry {
        Some(idx) => idx,
        None => {
            eprintln!("Baseline entry '{}' not found.", entry);
            process::exit(ExitCode::EntryNotFound as i32);
        }
    };

    let candidate_entry = match candidate_entry {
        Some(idx) => idx,
        None => {
            eprintln!("Candidate entry '{}' not found.", entry);
            process::exit(ExitCode::EntryNotFound as i32);
        }
    };

    let baseline_result = WcsdEngine::analyze(&baseline_graph, baseline_entry, usize::MAX);

    let candidate_result = WcsdEngine::analyze(&candidate_graph, candidate_entry, usize::MAX);

    let diff = WcsdDiffResult {
        baseline_depth_bytes: baseline_result.max_depth_bytes,
        candidate_depth_bytes: candidate_result.max_depth_bytes,
        stack_delta_bytes: candidate_result.max_depth_bytes as isize
            - baseline_result.max_depth_bytes as isize,
        baseline_unbounded: baseline_result.is_unbounded,
        candidate_unbounded: candidate_result.is_unbounded,
        new_recursion_introduced: !baseline_result.is_unbounded && candidate_result.is_unbounded,
        candidate_critical_path: candidate_result.critical_path.clone(),
    };

    match format {
        Format::Json => {
            let payload = serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "analysis_type": "diff",
                "result": diff
            });

            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        }

        Format::Sarif => {
            let mut results = vec![];

            if diff.new_recursion_introduced || diff.stack_delta_bytes > 0 {
                let message = if diff.new_recursion_introduced {
                    "New recursive stack cycle introduced.".to_string()
                } else {
                    format!(
                        "Stack regression introduced. Delta: {} bytes",
                        diff.stack_delta_bytes
                    )
                };

                let locations: Vec<_> = diff
                    .candidate_critical_path
                    .iter()
                    .map(|sym| {
                        serde_json::json!({
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": candidate
                                },
                                "region": {
                                    "snippet": {
                                        "text": sym
                                    }
                                }
                            }
                        })
                    })
                    .collect();

                results.push(serde_json::json!({
                    "ruleId": "STACK-002",
                    "level": "error",
                    "message": {
                        "text": message
                    },
                    "locations": locations
                }));
            }

            let sarif_payload = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
                "runs": [
                    {
                        "tool": {
                            "driver": {
                                "name": "StackScope",
                                "informationUri": "https://github.com/stackscope/stackscope",
                                "rules": [
                                    {
                                        "id": "STACK-002",
                                        "name": "StackRegressionIntroduced",
                                        "shortDescription": {
                                            "text": "Stack regression introduced."
                                        },
                                        "helpUri": "https://github.com/stackscope/stackscope/wiki/STACK-002"
                                    }
                                ]
                            }
                        },
                        "results": results
                    }
                ]
            });

            println!("{}", serde_json::to_string_pretty(&sarif_payload).unwrap());
        }

        Format::Text => {
            println!("=== Stack Regression Analysis ===");
            println!("Baseline Depth: {} bytes", diff.baseline_depth_bytes);
            println!("Candidate Depth: {} bytes", diff.candidate_depth_bytes);
            println!("Stack Delta: {} bytes", diff.stack_delta_bytes);

            if diff.new_recursion_introduced {
                println!("WARNING: New recursion introduced.");
            }
        }
    }

    if diff.new_recursion_introduced || diff.stack_delta_bytes > 0 {
        process::exit(ExitCode::PolicyViolation as i32);
    }

    process::exit(ExitCode::Success as i32);
}
