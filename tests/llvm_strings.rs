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
    native.matches(
        r#"
        print(""); print("日本語\0\r\n\t\"\\\u{1f600}");
        print("a\0x" != "a\0y"); print("a\0" == "a"); print("" == "");
        print("\u{e9}" == "e\u{301}"); print("日本語" == "\u{65e5}本語");
        print(42); print(true); print(1.5f64); print("end");
    "#,
        "\n日本語\0\r\n\t\"\\😀\ntrue\nfalse\ntrue\nfalse\ntrue\n42\ntrue\n1.5\nend\n",
    );
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
    native.matches(
        r#"
        type Label { id: i64, text: string = "既定", }
        fn identity(text: string) -> string { return text; }
        fn forward(text: string) -> string { return identity(text); }
        fn make() -> [[Label; 1]; 2] {
            mut rows: [[Label; 1]; 2] = [[Label { id: 0, }], [Label { id: 1, text: "保存", }]];
            saved: infer = rows;
            rows[1][0] = Label { id: 2, text: "変更", };
            print(rows[1][0].text);
            return saved;
        }
        fn replace(original: [string; 2]) -> [string; 2] {
            mut words: infer = original;
            words[0] = "replacement";
            return words;
        }
        fn main() -> void {
            original: infer = make();
            mut copy: infer = original;
            for (mut i: i64 = 0; i < 3; i = i + 1) {
                copy[1][0] = Label { id: 3, text: forward("\0end"), };
            }
            print(original[0][0].text); print(original[1][0].text); print(copy[1][0].text);
            mut text: string = "old"; saved: infer = text; text = "new";
            print(saved); print(text);
            words: [string; 2] = ["first", "second"];
            changed: infer = replace(words);
            print(words[0]); print(changed[0]); print(changed[1]);
        }
    "#,
        "変更\n既定\n保存\n\0end\nold\nnew\nfirst\nreplacement\nsecond\n",
    );
}

#[test]
fn effects_and_short_circuiting_keep_source_order() {
    let Some(native) = Native::new() else { return };
    native.matches(r#"
        fn mark(text: string) -> string { print(text); return text; }
        fn pair(left: string, right: string) -> string { return right; }
        fn index() -> i64 { print("index"); return 0; }
        fn fail() -> string { print(["bad"][1]); return "bad"; }
        type Pair { first: string = mark("default"), second: string, }
        print(mark("left") == mark("right"));
        print(pair(mark("arg1"), mark("arg2")));
        value: Pair = Pair { second: mark("explicit"), }; print(value.first);
        mut words: [string; 2] = [mark("item1"), mark("item2")];
        words[index()] = mark("replacement"); print(words[0]);
        print(false && fail() == "bad"); print(true || fail() != "bad");
        for (mut flag: bool = true; flag; flag = mark("update1") == mark("update2")) {
            if mark("condition1") != mark("condition2") { continue; }
        }
    "#, "left\nright\nfalse\narg1\narg2\narg2\nexplicit\ndefault\ndefault\nitem1\nitem2\nindex\nreplacement\nreplacement\nfalse\ntrue\ncondition1\ncondition2\nupdate1\nupdate2\n");
}

#[test]
fn string_array_bounds_still_fail_at_runtime() {
    let Some(native) = Native::new() else { return };
    for source in [
        "print([\"a\"][-1]);",
        "mut a: [string; 1] = [\"a\"]; a[1] = \"b\";",
    ] {
        assert!(run_vm(source).is_err());
        for optimization in ["-O0", "-O2"] {
            for llvm in [false, true] {
                assert!(!native.run(source, llvm, optimization).status.success());
            }
        }
    }
}
