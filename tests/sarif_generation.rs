use std::process::Command;

#[test]
fn generates_sarif_output() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let elf_path = format!("{}/fixtures/cortex_m_target/target.elf", manifest_dir);

    let output = Command::new("cargo")
        .args([
            "run", "--", "analyze", "--elf", &elf_path, "--entry", "main", "--budget", "100",
            "--format", "sarif",
        ])
        .output()
        .expect("failed to execute StackScope");

    // Policy violation is expected for recursive fixture
    assert_eq!(output.status.code(), Some(1));

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("\"version\": \"2.1.0\""),
        "SARIF version missing"
    );

    assert!(
        stdout.contains("\"ruleId\": \"STACK-001\""),
        "Expected STACK-001 rule missing"
    );

    assert!(
        stdout.contains("recursive cycle"),
        "Expected recursive cycle message missing"
    );
}
