#[path = "support/string_cases.rs"]
mod string_cases;

use primer_lang::{
    codegen::qbe::Target, compile_to_qbe, compile_to_qbe_with_target, compile_to_wat,
    compile_to_x86_64_win_asm, run_vm,
};
use std::{ffi::OsString, fs, path::PathBuf, process::Command};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "primer-string-routes-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        // このテスト自身が新規作成したディレクトリだけを片付けます。
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn tool(variable: &str, default: &str, version: &str) -> Option<OsString> {
    let configured = std::env::var_os(variable);
    let name = configured.clone().unwrap_or_else(|| default.into());
    if Command::new(&name)
        .arg(version)
        .output()
        .is_ok_and(|o| o.status.success())
    {
        Some(name)
    } else {
        assert!(configured.is_none(), "{variable} is unavailable: {name:?}");
        eprintln!(
            "execution comparison skipped: {name:?} unavailable; set {variable} to require it"
        );
        None
    }
}

fn run_success(command: &mut Command) -> Vec<u8> {
    let output = command.output().expect("start validation command");
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn cases() -> Vec<(&'static str, String)> {
    let mut cases: Vec<_> = string_cases::CASES
        .iter()
        .map(|&(s, e)| (s, e.to_owned()))
        .collect();
    cases.push((string_cases::UNUSED_DEFAULT, "1\ntrue\n".into()));
    for source in [
        include_str!("../examples/string_values.prim"),
        include_str!("../examples/string_lookup.prim"),
    ] {
        cases.push((source, run_vm(source).unwrap()));
    }
    cases
}

#[test]
fn every_route_generates_deterministic_artifacts() {
    for (source, expected) in cases() {
        assert_eq!(run_vm(source).unwrap(), expected);
        for emit in [compile_to_wat, compile_to_x86_64_win_asm] {
            assert_eq!(emit(source).unwrap(), emit(source).unwrap());
        }
        let qbe = compile_to_qbe_with_target(source, Some(Target::X86_64UnknownLinuxGnu)).unwrap();
        assert_eq!(
            qbe,
            compile_to_qbe_with_target(source, Some(Target::X86_64UnknownLinuxGnu)).unwrap()
        );
        assert!(qbe.starts_with("# target: x86_64-unknown-linux-gnu"));
        assert!(qbe.contains("loadub"));
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("\"write_byte\"") && wat.contains("i32.load8_u"));
        assert!(
            !wat.contains("(export \"memory\"") && !wat.contains("(import \"primer\" \"memory\"")
        );
        let asm = compile_to_x86_64_win_asm(source).unwrap();
        assert!(asm.contains("callq _setmode") && asm.contains("movzbl"));
    }
}

#[test]
fn qbe_requires_an_explicit_runtime_target() {
    for source in [
        "print(\"x\");",
        "type Item { text: string, }",
        "fn unused(text: string) -> string { return text; }",
    ] {
        let error = compile_to_qbe(source).unwrap_err();
        assert!(error.message().contains("explicit --target"));
        assert!(error.primary_span().is_some());
        assert!(compile_to_qbe_with_target(source, Some(Target::X86_64UnknownLinuxGnu)).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }
    let workspace = Workspace::new();
    let input = workspace.0.join("input.prim");
    let output = workspace.0.join("output.ssa");
    fs::write(&input, "print(\"x\");").unwrap();
    for options in [
        vec![],
        vec!["--target"],
        vec!["--target", "x86_64-pc-windows-msvc"],
        vec!["--target", "unknown"],
    ] {
        fs::write(&output, b"existing artifact").unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_primer"))
            .arg("emit-qbe")
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .args(options)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(!result.stderr.is_empty());
        assert_eq!(fs::read(&output).unwrap(), b"existing artifact");
    }
}

#[test]
fn direct_assembly_matches_known_bytes_and_vm_on_windows() {
    if !cfg!(all(
        target_os = "windows",
        target_arch = "x86_64",
        target_env = "msvc"
    )) {
        eprintln!("direct assembly execution skipped: requires Windows x64 with MSVC CRT");
        return;
    }
    let Some(clang) = tool("PRIMER_TEST_ASM_CLANG", "clang", "--version") else {
        return;
    };
    let workspace = Workspace::new();
    let assembly = workspace.0.join("program.s");
    let executable = workspace.0.join("program.exe");
    for (source, expected) in cases() {
        fs::write(&assembly, compile_to_x86_64_win_asm(source).unwrap()).unwrap();
        run_success(
            Command::new(&clang)
                .arg("--target=x86_64-pc-windows-msvc")
                .arg(&assembly)
                .arg("-o")
                .arg(&executable),
        );
        assert_eq!(
            run_success(&mut Command::new(&executable)),
            expected.as_bytes(),
            "{source}"
        );
    }
    for source in string_cases::OUT_OF_BOUNDS {
        assert!(run_vm(source).is_err());
        fs::write(&assembly, compile_to_x86_64_win_asm(source).unwrap()).unwrap();
        run_success(
            Command::new(&clang)
                .arg("--target=x86_64-pc-windows-msvc")
                .arg(&assembly)
                .arg("-o")
                .arg(&executable),
        );
        let failed = Command::new(&executable)
            .current_dir(&workspace.0)
            .output()
            .unwrap();
        assert!(!failed.status.success());
        assert!(failed.stdout.is_empty());
    }
}

#[test]
fn qbe_matches_known_bytes_and_vm_on_linux() {
    if !cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
        eprintln!("QBE execution skipped: requires Linux x86-64");
        return;
    }
    let Some(qbe) = tool("PRIMER_TEST_QBE", "qbe", "-h") else {
        return;
    };
    let Some(cc) = tool("PRIMER_TEST_CC", "cc", "--version") else {
        return;
    };
    let workspace = Workspace::new();
    let input = workspace.0.join("program.ssa");
    let assembly = workspace.0.join("program.s");
    let executable = workspace.0.join("program");
    for (source, expected) in cases() {
        fs::write(
            &input,
            compile_to_qbe_with_target(source, Some(Target::X86_64UnknownLinuxGnu)).unwrap(),
        )
        .unwrap();
        run_success(
            Command::new(&qbe)
                .args(["-t", "amd64_sysv"])
                .arg("-o")
                .arg(&assembly)
                .arg(&input),
        );
        run_success(Command::new(&cc).arg(&assembly).arg("-o").arg(&executable));
        assert_eq!(
            run_success(&mut Command::new(&executable)),
            expected.as_bytes(),
            "{source}"
        );
    }
    for source in string_cases::OUT_OF_BOUNDS {
        assert!(run_vm(source).is_err());
        fs::write(
            &input,
            compile_to_qbe_with_target(source, Some(Target::X86_64UnknownLinuxGnu)).unwrap(),
        )
        .unwrap();
        run_success(
            Command::new(&qbe)
                .args(["-t", "amd64_sysv"])
                .arg("-o")
                .arg(&assembly)
                .arg(&input),
        );
        run_success(Command::new(&cc).arg(&assembly).arg("-o").arg(&executable));
        let failed = Command::new(&executable)
            .current_dir(&workspace.0)
            .output()
            .unwrap();
        assert!(!failed.status.success());
        assert!(failed.stdout.is_empty());
    }
}

#[test]
fn wat_matches_known_bytes_and_vm_without_exposing_memory() {
    let Some(node) = tool("PRIMER_TEST_NODE", "node", "--version") else {
        return;
    };
    // WABTのJS版をNodeで呼び、Windowsでも同じ変換ツールを使います。
    let wabt = std::env::var_os("PRIMER_TEST_WAT2WASM_JS")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/wasm-tools/node_modules/wabt/bin/wat2wasm")
        });
    if !wabt.is_file() {
        assert!(
            std::env::var_os("PRIMER_TEST_WAT2WASM_JS").is_none(),
            "configured WABT is unavailable"
        );
        eprintln!("WAT execution skipped: set PRIMER_TEST_WAT2WASM_JS to WABT's bin/wat2wasm");
        return;
    }
    let workspace = Workspace::new();
    let wat = workspace.0.join("program.wat");
    let wasm = workspace.0.join("program.wasm");
    let host = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/run_wasm.cjs");
    for (source, expected) in cases() {
        fs::write(&wat, compile_to_wat(source).unwrap()).unwrap();
        run_success(
            Command::new(&node)
                .arg(&wabt)
                .arg(&wat)
                .arg("-o")
                .arg(&wasm),
        );
        assert_eq!(
            run_success(Command::new(&node).arg(&host).arg(&wasm)),
            expected.as_bytes(),
            "{source}"
        );
    }
    for source in string_cases::OUT_OF_BOUNDS {
        assert!(run_vm(source).is_err());
        fs::write(&wat, compile_to_wat(source).unwrap()).unwrap();
        run_success(
            Command::new(&node)
                .arg(&wabt)
                .arg(&wat)
                .arg("-o")
                .arg(&wasm),
        );
        let failed = Command::new(&node).arg(&host).arg(&wasm).output().unwrap();
        assert!(!failed.status.success());
        assert!(failed.stdout.is_empty());
        assert!(String::from_utf8_lossy(&failed.stderr).contains("unreachable"));
    }
}
