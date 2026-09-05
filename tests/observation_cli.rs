use std::{fs, path::PathBuf, process::Command};

const OBSERVATION_CASES: &[&str] = &[
    "i64-addition",
    "checked-i64-arithmetic",
    "i64-minimum-literal",
    "integer-conversions",
    "fixed-width-integers",
    "small-integers",
    "integer-bit-operations",
    "numeric-conversions",
    "float-types",
    "float-output",
    "mutable-assignment",
    "boolean-comparisons",
    "short-circuit",
    "conditional-scope",
    "while-loop",
    "loop-control",
    "for-loop",
    "product-types",
    "functions",
    "aggregate-functions",
    "fixed-arrays",
    "product-array-field",
    "product-array-elements",
    "nested-fixed-arrays",
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
    let mut process = Command::new(env!("CARGO_BIN_EXE_primer"));
    process.arg(command).arg(source_path(case_name));
    if matches!(command, "emit-llvm" | "emit-qbe")
        && matches!(case_name, "string-values" | "string-byte-length")
    {
        process.args(["--target", "x86_64-unknown-linux-gnu"]);
    }
    let output = process
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
fn string_observations_match_ir_bytecode_and_vm_output() {
    // 同じ文字列ソースから全経路の変換結果を固定します。
    for (command, file) in [
        ("emit-ir", "ir.pir"),
        ("emit-c", "c.c"),
        ("emit-llvm", "llvm.ll"),
        ("emit-qbe", "qbe.ssa"),
        ("emit-wat", "wat.wat"),
        ("emit-asm", "asm.s"),
        ("emit-bytecode", "bytecode.pbc"),
        ("run", "run.stdout"),
    ] {
        assert_observation("string-values", command, file);
        assert_observation("string-byte-length", command, file);
    }
}

#[test]
fn missing_string_targets_fail_without_overwriting_an_artifact() {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "primer-string-output-{}-{id}.tmp",
        std::process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    use std::io::Write;
    file.write_all(b"existing artifact").unwrap();
    drop(file);
    for command in ["emit-llvm", "emit-qbe"] {
        let output = Command::new(env!("CARGO_BIN_EXE_primer"))
            .arg(command)
            .arg(source_path("string-values"))
            .arg("-o")
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let message = String::from_utf8(output.stderr).unwrap();
        assert!(
            message.contains("explicit --target") && message.contains("at 1:"),
            "{message}"
        );
        assert_eq!(fs::read(&path).unwrap(), b"existing artifact");
    }
    fs::remove_file(path).unwrap();
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
