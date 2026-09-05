#[path = "support/string_cases.rs"]
mod string_cases;

use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use primer_lang::{
    codegen::llvm::Target, compile_to_c, compile_to_llvm, compile_to_llvm_with_target, run_vm,
};

const LINUX: Target = Target::X86_64UnknownLinuxGnu;
const WINDOWS: Target = Target::X86_64PcWindowsMsvc;

#[test]
fn target_is_explicit_and_emission_is_deterministic() {
    let source = "print(\"日\\0\\r\\n\");";
    let error = compile_to_llvm(source).unwrap_err();
    assert!(error.message().contains("explicit --target"));
    assert!(error.primary_span().is_some());
    for target in [LINUX, WINDOWS] {
        let llvm = compile_to_llvm_with_target(source, Some(target)).unwrap();
        assert!(llvm.starts_with(&format!("target triple = \"{}\"", target.triple())));
        assert!(llvm.contains("%primer.string = type { ptr, i64 }"));
        assert!(llvm.contains("[6 x i8] c\"\\E6\\97\\A5\\00\\0D\\0A\""));
        assert_eq!(llvm.contains("@_setmode"), target == WINDOWS);
        assert!(llvm.contains("zext i8 %byte to i32"));
        assert!(
            !llvm.contains("@malloc") && !llvm.contains("@strlen") && !llvm.contains("@strcmp")
        );
        assert_eq!(
            llvm,
            compile_to_llvm_with_target(source, Some(target)).unwrap()
        );
    }
    for target in [None, Some(LINUX), Some(WINDOWS)] {
        let llvm = compile_to_llvm_with_target("print(1);", target).unwrap();
        assert!(!llvm.contains("@_setmode") && !llvm.contains("%primer.string"));
        assert_eq!(llvm.contains("target triple"), target.is_some());
    }
}

#[test]
fn unused_string_types_and_functions_require_a_target() {
    for source in [
        "type Label { text: string, }",
        "type Rows { values: [[string; 1]; 2], }",
        "fn identity(value: string) -> string { return value; }",
        "fn unused() -> string { return \"unused\"; }",
        "fn unused() -> bool { return \"a\" == \"b\"; }",
    ] {
        assert!(
            compile_to_llvm(source)
                .unwrap_err()
                .primary_span()
                .is_some()
        );
        for target in [LINUX, WINDOWS] {
            let llvm = compile_to_llvm_with_target(source, Some(target)).unwrap();
            assert!(llvm.contains("%primer.string = type"), "{source}");
        }
    }
}

#[test]
fn cli_validates_targets_before_overwriting_output() {
    let directory = test_directory();
    let source = directory.join("source.prim");
    let artifact = directory.join("output.ll");
    fs::write(&source, "print(\"test\");").unwrap();
    for flags in [
        vec![],
        vec!["--target"],
        vec!["--target", "unknown"],
        vec!["--target", LINUX.triple(), "--target", WINDOWS.triple()],
    ] {
        fs::write(&artifact, b"existing artifact").unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_primer"))
            .arg("emit-llvm")
            .arg(&source)
            .arg("-o")
            .arg(&artifact)
            .args(flags)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
        assert_eq!(fs::read(&artifact).unwrap(), b"existing artifact");
    }
    for target_first in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_primer"));
        command.arg("emit-llvm").arg(&source);
        if target_first {
            command.args(["--target", LINUX.triple()]);
        }
        command.arg("--output").arg(&artifact);
        if !target_first {
            command.args(["--target", LINUX.triple()]);
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(
            fs::read_to_string(&artifact)
                .unwrap()
                .contains(LINUX.triple())
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

fn test_directory() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "primer-llvm-strings-{}-{stamp}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    directory
}

struct Native {
    directory: PathBuf,
    clang: OsString,
    cc: OsString,
    target: Target,
}

impl Native {
    fn new() -> Option<Self> {
        // 実行テスト側がホストに合うターゲットを指定します。生成APIは推測しません。
        let target = if cfg!(all(
            target_os = "windows",
            target_arch = "x86_64",
            target_env = "msvc"
        )) {
            WINDOWS
        } else if cfg!(all(
            target_os = "linux",
            target_arch = "x86_64",
            target_env = "gnu"
        )) {
            LINUX
        } else {
            assert!(
                std::env::var_os("PRIMER_TEST_LLVM_CLANG").is_none(),
                "native LLVM test target unavailable"
            );
            eprintln!("native LLVM execution skipped: unsupported test host");
            return None;
        };
        fn compiler(variable: &str, default: &str) -> Option<OsString> {
            let configured = std::env::var_os(variable);
            let compiler = configured.clone().unwrap_or_else(|| default.into());
            if Command::new(&compiler)
                .arg("--version")
                .output()
                .is_ok_and(|o| o.status.success())
            {
                Some(compiler)
            } else {
                assert!(
                    configured.is_none(),
                    "{variable} compiler unavailable: {compiler:?}"
                );
                eprintln!(
                    "native comparison skipped: {compiler:?} unavailable; set {variable} to require it"
                );
                None
            }
        }
        let clang = compiler("PRIMER_TEST_LLVM_CLANG", "clang")?;
        let cc = compiler("PRIMER_TEST_CC", if cfg!(windows) { "clang" } else { "cc" })?;
        Some(Self {
            directory: test_directory(),
            clang,
            cc,
            target,
        })
    }

    fn run(&self, source: &str, llvm: bool, optimization: &str) -> Output {
        let input = self
            .directory
            .join(if llvm { "program.ll" } else { "program.c" });
        let executable = self.directory.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        fs::write(
            &input,
            if llvm {
                compile_to_llvm_with_target(source, Some(self.target)).unwrap()
            } else {
                compile_to_c(source).unwrap()
            },
        )
        .unwrap();
        let mut command = Command::new(if llvm { &self.clang } else { &self.cc });
        command.arg(optimization);
        if llvm {
            command.arg(format!("--target={}", self.target.triple()));
        } else {
            command.args(["-std=c11", "-pedantic-errors"]);
        }
        command.arg(&input).arg("-o").arg(&executable);
        if !cfg!(windows) {
            command.arg("-lm");
        }
        let built = command.output().expect("start native compiler");
        assert!(
            built.status.success(),
            "LLVM={llvm} {optimization}: {}\n{source}",
            String::from_utf8_lossy(&built.stderr)
        );
        Command::new(&executable)
            .current_dir(&self.directory)
            .output()
            .expect("run generated program")
    }

    fn matches(&self, source: &str, expected: &str) {
        assert_eq!(run_vm(source).unwrap(), expected);
        for optimization in ["-O0", "-O2"] {
            for llvm in [false, true] {
                let actual = self.run(source, llvm, optimization);
                assert!(
                    actual.status.success(),
                    "LLVM={llvm}: {}",
                    String::from_utf8_lossy(&actual.stderr)
                );
                assert!(actual.stderr.is_empty(), "{:?}", actual.stderr);
                // 改行・文字コードを正規化せず、既知の期待値、VM、C、LLVMを比較します。
                assert_eq!(
                    actual.stdout,
                    expected.as_bytes(),
                    "LLVM={llvm} {optimization}\n{source}"
                );
            }
        }
    }
}

impl Drop for Native {
    fn drop(&mut self) {
        // このテストが新規作成した専用ディレクトリだけを削除します。
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn bytes_equality_and_mixed_output_match_vm_and_c() {
    let Some(native) = Native::new() else { return };
    native.matches(string_cases::CASES[0].0, string_cases::CASES[0].1);
    native.matches(string_cases::UNUSED_DEFAULT, "1\ntrue\n");
    for source in [
        include_str!("../examples/string_values.prim"),
        include_str!("../examples/string_lookup.prim"),
    ] {
        native.matches(source, &run_vm(source).unwrap());
    }
}

#[test]
fn return_values_arrays_products_and_copies_remain_independent() {
    let Some(native) = Native::new() else { return };
    native.matches(string_cases::CASES[1].0, string_cases::CASES[1].1);
}

#[test]
fn effects_and_short_circuiting_keep_source_order() {
    let Some(native) = Native::new() else { return };
    native.matches(string_cases::CASES[2].0, string_cases::CASES[2].1);
}

#[test]
fn string_array_bounds_still_fail_at_runtime() {
    let Some(native) = Native::new() else { return };
    for source in string_cases::OUT_OF_BOUNDS {
        assert!(run_vm(source).is_err());
        for optimization in ["-O0", "-O2"] {
            for llvm in [false, true] {
                assert!(!native.run(source, llvm, optimization).status.success());
            }
        }
    }
}
