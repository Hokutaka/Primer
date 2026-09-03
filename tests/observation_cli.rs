use std::{fs, path::PathBuf, process::Command};

const OBSERVATION_CASES: &[&str] = &[
    "i64-addition",
    "float-types",
    "mutable-assignment",
    "boolean-comparisons",
    "conditional-scope",
    "while-loop",
    "loop-control",
    "for-loop",
    "product-types",
];

fn fixture_path(case_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("observation")
        .join(case_name)
}

fn source_path(case_name: &str) -> PathBuf {
    fixture_path(case_name).join("source.prim")
}

fn expected_output(case_name: &str, file_name: &str) -> String {
    let path = fixture_path(case_name).join("expected").join(file_name);

    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn assert_observation(case_name: &str, command: &str, expected_file: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_primer"))
        .arg(command)
        .arg(source_path(case_name))
        .output()
        .unwrap_or_else(|error| panic!("failed to run primer {command}: {error}"));

    assert!(
        output.status.success(),
        "primer {command} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output.stderr.is_empty(),
        "primer {command} wrote to stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("primer {command} emitted non-UTF-8 output: {error}"));

    let expected = expected_output(case_name, expected_file);

    // Gitのチェックアウト設定に左右されないように、期待値の改行コードを LF に揃える
    let expected = expected.replace("\r\n", "\n");

    assert_eq!(
        actual, expected,
        "unexpected output from primer {command} for case `{case_name}`"
    );
}

fn assert_observation_cases(command: &str, expected_file: &str) {
    for &case_name in OBSERVATION_CASES {
        assert_observation(case_name, command, expected_file);
    }
}

#[test]
fn emit_ir_matches_expected_output() {
    assert_observation_cases("emit-ir", "ir.pir");
}

#[test]
fn emit_c_matches_expected_output() {
    assert_observation_cases("emit-c", "c.c");
}

#[test]
fn emit_llvm_matches_expected_output() {
    assert_observation_cases("emit-llvm", "llvm.ll");
}

#[test]
fn emit_qbe_matches_expected_output() {
    assert_observation_cases("emit-qbe", "qbe.ssa");
}

#[test]
fn emit_wat_matches_expected_output() {
    assert_observation_cases("emit-wat", "wat.wat");
}

#[test]
fn emit_asm_matches_expected_output() {
    assert_observation_cases("emit-asm", "asm.s");
}

#[test]
fn emit_bytecode_matches_expected_output() {
    assert_observation_cases("emit-bytecode", "bytecode.pbc");
}

#[test]
fn run_matches_expected_output() {
    assert_observation_cases("run", "run.stdout");
}
