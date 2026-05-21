use serde::Serialize;

use crate::sdt::abstract_interpreter::{WcsdEngine, WcsdResult};
use crate::types::ExecutionGraph;

use petgraph::graph::NodeIndex;

#[derive(Debug, Serialize)]
pub struct WcsdDiffResult {
    pub baseline_depth_bytes: usize,
    pub candidate_depth_bytes: usize,

    /// Positive means regression.
    pub stack_delta_bytes: isize,

    pub baseline_unbounded: bool,
    pub candidate_unbounded: bool,

    /// Did the candidate introduce recursion?
    pub new_recursion_introduced: bool,

    pub candidate_critical_path: Vec<String>,
}

#[allow(dead_code)]
pub struct DiffEngine;

#[allow(dead_code)]
impl DiffEngine {
    pub fn diff(
        baseline_graph: &ExecutionGraph,
        candidate_graph: &ExecutionGraph,
        baseline_entry: NodeIndex,
        candidate_entry: NodeIndex,
        budget: usize,
    ) -> WcsdDiffResult {
        let baseline_result = WcsdEngine::analyze(baseline_graph, baseline_entry, budget);

        let candidate_result = WcsdEngine::analyze(candidate_graph, candidate_entry, budget);

        Self::build_diff(baseline_result, candidate_result)
    }

    fn build_diff(baseline: WcsdResult, candidate: WcsdResult) -> WcsdDiffResult {
        WcsdDiffResult {
            baseline_depth_bytes: baseline.max_depth_bytes,

            candidate_depth_bytes: candidate.max_depth_bytes,

            stack_delta_bytes: candidate.max_depth_bytes as isize
                - baseline.max_depth_bytes as isize,

            baseline_unbounded: baseline.is_unbounded,

            candidate_unbounded: candidate.is_unbounded,

            new_recursion_introduced: !baseline.is_unbounded && candidate.is_unbounded,

            candidate_critical_path: candidate.critical_path,
        }
    }
}
