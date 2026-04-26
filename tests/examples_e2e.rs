use std::process::Command;

fn cargo_cmd() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn run_example(example: &str) -> (String, String, bool) {
    let output = Command::new(cargo_cmd())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("run")
        .arg("--quiet")
        .arg("--example")
        .arg(example)
        .output()
        .expect("failed to execute cargo run --example");

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

#[test]
fn e2e_examples_run_successfully() {
    let cases = [
        ("run_wasm", "f() -> [42]"),
        ("host_imports", "call_add() -> 7"),
        ("memory_io", "u32 at memory[0..4] after invoke: 42"),
        ("wasi_config", "f() with WASI config -> 42"),
    ];

    for (example, expected_output) in cases {
        let (stdout, stderr, ok) = run_example(example);
        assert!(
            ok,
            "example {example} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            stdout.contains(expected_output),
            "example {example} missing expected output: {expected_output}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}
