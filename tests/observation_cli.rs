use std::{path::PathBuf, process::Command};

fn source_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("observation")
        .join("source.prim")
}

fn assert_observation(command: &str, expected: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_primer"))
        .arg(command)
        .arg(source_path())
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

    // Gitのチェックアウト設定に左右されないように、期待値の改行コードを LF に揃える
    let expected = expected.replace("\r\n", "\n");
    assert_eq!(actual, expected, "unexpected output from primer {command}");
}

#[test]
fn emit_ir_matches_expected_output() {
    assert_observation(
        "emit-ir",
        include_str!("fixtures/observation/expected/ir.pir.txt"),
    );
}

#[test]
fn emit_c_matches_expected_output() {
    assert_observation(
        "emit-c",
        include_str!("fixtures/observation/expected/c.c.txt"),
    );
}

#[test]
fn emit_llvm_matches_expected_output() {
    assert_observation(
        "emit-llvm",
        include_str!("fixtures/observation/expected/llvm.ll.txt"),
    );
}

#[test]
fn emit_qbe_matches_expected_output() {
    assert_observation(
        "emit-qbe",
        include_str!("fixtures/observation/expected/qbe.ssa.txt"),
    );
}

#[test]
fn emit_wat_matches_expected_output() {
    assert_observation(
        "emit-wat",
        include_str!("fixtures/observation/expected/wat.wat.txt"),
    );
}

#[test]
fn emit_asm_matches_expected_output() {
    assert_observation(
        "emit-asm",
        include_str!("fixtures/observation/expected/asm.s.txt"),
    );
}

#[test]
fn emit_bytecode_matches_expected_output() {
    assert_observation(
        "emit-bytecode",
        include_str!("fixtures/observation/expected/bytecode.pbc.txt"),
    );
}

#[test]
fn run_matches_expected_output() {
    assert_observation("run", "3\n");
}
