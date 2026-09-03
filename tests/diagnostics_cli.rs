use std::{fs, path::PathBuf, process::Command};

fn fixture_path(case_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("diagnostics")
        .join(case_name)
}

fn expected_output(case_name: &str, file_name: &str) -> String {
    let path = fixture_path(case_name).join("expected").join(file_name);

    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn assert_diagnostic(case_name: &str, command: &str, expected_file: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_primer"))
        .arg(command)
        .arg(fixture_path(case_name).join("source.prim"))
        .output()
        .unwrap_or_else(|error| panic!("failed to run primer {command}: {error}"));

    assert_eq!(
        output.status.code(),
        Some(1),
        "primer {command} returned an unexpected status for case `{case_name}`"
    );

    assert!(
        output.stdout.is_empty(),
        "primer {command} wrote to stdout for case `{case_name}`:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let actual = String::from_utf8(output.stderr)
        .unwrap_or_else(|error| panic!("primer {command} emitted non-UTF-8 stderr: {error}"));

    let expected = expected_output(case_name, expected_file);

    // OSやGitの設定による改行コードの違いを診断内容の差として扱わない
    assert_eq!(
        normalize_line_endings(&actual),
        normalize_line_endings(&expected),
        "unexpected stderr from primer {command} for case `{case_name}`"
    );
}

#[test]
fn lexer_unexpected_character_matches_expected_diagnostic() {
    assert_diagnostic("lexer-unexpected-character", "check", "check.stderr");
}

#[test]
fn parser_missing_semicolon_matches_expected_diagnostic() {
    assert_diagnostic("parser-missing-semicolon", "check", "check.stderr");
}

#[test]
fn semantic_unknown_binding_matches_expected_diagnostic() {
    assert_diagnostic("semantic-unknown-binding", "check", "check.stderr");
}

#[test]
fn semantic_immutable_assignment_matches_expected_diagnostic() {
    assert_diagnostic("semantic-immutable-assignment", "check", "check.stderr");
}

#[test]
fn semantic_boolean_arithmetic_matches_expected_diagnostic() {
    assert_diagnostic("semantic-boolean-arithmetic", "check", "check.stderr");
}

#[test]
fn semantic_if_condition_matches_expected_diagnostic() {
    assert_diagnostic("semantic-if-condition", "check", "check.stderr");
}

#[test]
fn semantic_while_condition_matches_expected_diagnostic() {
    assert_diagnostic("semantic-while-condition", "check", "check.stderr");
}

#[test]
fn semantic_for_condition_matches_expected_diagnostic() {
    assert_diagnostic("semantic-for-condition", "check", "check.stderr");
}

#[test]
fn semantic_for_scope_matches_expected_diagnostic() {
    assert_diagnostic("semantic-for-scope", "check", "check.stderr");
}

#[test]
fn semantic_break_outside_loop_matches_expected_diagnostic() {
    assert_diagnostic("semantic-break-outside-loop", "check", "check.stderr");
}

#[test]
fn semantic_continue_outside_loop_matches_expected_diagnostic() {
    assert_diagnostic("semantic-continue-outside-loop", "check", "check.stderr");
}

#[test]
fn semantic_block_scope_matches_expected_diagnostic() {
    assert_diagnostic("semantic-block-scope", "check", "check.stderr");
}

#[test]
fn emit_ir_semantic_unknown_binding_matches_expected_diagnostic() {
    assert_diagnostic("semantic-unknown-binding", "emit-ir", "emit-ir.stderr");
}

#[test]
fn run_compilation_error_matches_expected_diagnostic() {
    assert_diagnostic("semantic-unknown-binding", "run", "run.stderr");
}

#[test]
fn run_vm_error_matches_expected_diagnostic() {
    assert_diagnostic("vm-division-by-zero", "run", "run.stderr");
}
