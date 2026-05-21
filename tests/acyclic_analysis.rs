use std::process::Command;

#[test]
fn acyclic_firmware_has_bounded_stack() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--elf",
            "fixtures/cortex_m_target_acyclic/target.elf",
            "--entry",
            "main",
            "--budget",
            "512",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute stackscope");

    assert!(
        output.status.success(),
        "expected success exit code, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("\"is_unbounded\": false"));
    assert!(stdout.contains("\"budget_exceeded\": false"));
    assert!(stdout.contains("\"max_depth_bytes\": 148"));

    assert!(stdout.contains("function_a"));
    assert!(stdout.contains("function_b"));
    assert!(stdout.contains("function_c"));
}
