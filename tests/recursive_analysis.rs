use std::process::Command;

#[test]
fn recursive_firmware_triggers_policy_violation() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--elf",
            "fixtures/cortex_m_target/target.elf",
            "--entry",
            "main",
            "--budget",
            "100",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute stackscope");

    assert_eq!(output.status.code(), Some(1));

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("\"is_unbounded\": true"));
    assert!(stdout.contains("\"budget_exceeded\": true"));
    assert!(stdout.contains("recursive_function"));
}
