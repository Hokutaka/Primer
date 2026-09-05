use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use primer_lang::{compile_to_c, run_vm};

struct NativeC {
    directory: PathBuf,
    compiler: OsString,
}

impl NativeC {
    fn new() -> Option<Self> {
        let configured = std::env::var_os("PRIMER_TEST_CC");
        let compiler = configured
            .clone()
            .unwrap_or_else(|| if cfg!(windows) { "clang" } else { "cc" }.into());
        let available = Command::new(&compiler)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if !available {
            assert!(
                configured.is_none(),
                "PRIMER_TEST_CC compiler is unavailable: {compiler:?}"
            );
            eprintln!(
                "native C execution skipped: {compiler:?} unavailable; set PRIMER_TEST_CC to require it"
            );
            return None;
        }
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "primer-c-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Some(Self {
            directory,
            compiler,
        })
    }

    fn run(&self, source: &str, optimization: &str) -> Output {
        let c = self.directory.join("program.c");
        let exe = self.directory.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        fs::write(&c, compile_to_c(source).unwrap()).unwrap();
        let mut command = Command::new(&self.compiler);
        command.args(["-std=c11", "-pedantic-errors", optimization]);
        if std::env::var_os("PRIMER_TEST_SANITIZE").is_some() {
            command.args(["-fsanitize=address,undefined", "-fno-omit-frame-pointer"]);
        }
        command.arg(&c).arg("-o").arg(&exe);
        if !cfg!(windows) {
            command.arg("-lm");
        }
        let built = command.output().expect("start C compiler");
        assert!(
            built.status.success(),
            "C compile failed:\n{}\n{source}",
            String::from_utf8_lossy(&built.stderr)
        );
        Command::new(exe)
            .current_dir(&self.directory)
            .output()
            .expect("run generated C")
    }

    fn matches_vm(&self, source: &str) {
        let expected = run_vm(source).unwrap();
        for optimization in ["-O0", "-O2"] {
            let actual = self.run(source, optimization);
            assert!(
                actual.status.success(),
                "C run failed: {}\n{source}",
                String::from_utf8_lossy(&actual.stderr)
            );
            assert!(
                actual.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&actual.stderr)
            );
            // 改行の正規化をせず、NULを含む出力バイト全体を比較します。
            assert_eq!(
                actual.stdout,
                expected.as_bytes(),
                "{optimization}\n{source}"
            );
        }
    }
}

impl Drop for NativeC {
    fn drop(&mut self) {
        // このテスト自身がcreate_dirで作成した専用ディレクトリだけを削除します。
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn literals_emit_ascii_c_and_explicit_utf8_lengths() {
    let source = r#"print("日本語\n\0\u{1f600}\"\\??/");"#;
    let c = compile_to_c(source).unwrap();
    assert!(c.is_ascii());
    assert!(c.contains("const unsigned char *data;"));
    assert!(c.contains("size_t length;"));
    assert!(c.contains("\\346\\227\\245"));
    assert!(c.contains("fwrite(value.data, 1, value.length, stdout)"));
    assert!(c.contains("memcmp(left.data, right.data, left.length)"));
    assert!(
        !c.contains("malloc(")
            && !c.contains("free(")
            && !c.contains("strlen(")
            && !c.contains("strcmp(")
    );
    assert_eq!(c, compile_to_c(source).unwrap());
}

#[test]
fn strings_and_examples_match_vm_as_exact_bytes() {
    let Some(c) = NativeC::new() else { return };
    for source in [
        include_str!("../examples/string_values.prim"),
        include_str!("../examples/string_lookup.prim"),
        r#"print(""); print("a\0b\n\r\t\"\\??/9\u{1f600}");
            print("\u{e9}" == "e\u{301}"); print("a\0x" != "a\0y");
            print("a\0" == "a"); print("" == ""); print("日本語" == "\u{65e5}本語");
            if "a" != "b" { print("different"); }
            while "same" == "same" { print("same"); break; }"#,
    ] {
        c.matches_vm(source);
    }
}

#[test]
fn returned_strings_survive_calls_loops_and_aggregate_copies() {
    let Some(c) = NativeC::new() else { return };
    c.matches_vm(
        r#"
        type Label { id: i64, text: string = "既定", }
        fn make() -> [[Label; 1]; 2] {
            mut values: [[Label; 1]; 2] = [[Label { id: 0, }], [Label { id: 1, text: "保存", }]];
            saved: infer = values;
            values[1][0] = Label { id: 2, text: "変更", };
            print(values[1][0].text);
            return saved;
        }
        fn main() -> void {
            original: infer = make();
            mut copy: infer = original;
            for (mut i: i64 = 0; i < 3; i = i + 1) {
                words: [string; 2] = ["loop", "\0end"];
                copy[1][0] = Label { id: 3, text: words[1], };
            }
            print(original[0][0].text);
            print(original[1][0].text);
            print(copy[1][0].text);
        }
    "#,
    );
}

#[test]
fn c_preserves_call_comparison_construction_and_index_order() {
    let Some(c) = NativeC::new() else { return };
    c.matches_vm(
        r#"
        fn mark(text: string) -> string { print(text); return text; }
        fn pair(left: string, right: string) -> string { print(left); return right; }
        fn show(left: string, right: string) -> void { print(left); print(right); }
        fn array() -> [string; 2] { print("base"); return ["zero", "one"]; }
        fn index() -> i64 { print("index"); return 1; }
        type Pair { first: string = mark("default"), second: string, }
        print(mark("left") == mark("right"));
        print(pair(mark("arg1"), mark("arg2")));
        show(mark("void1"), mark("void2"));
        value: Pair = Pair { second: mark("explicit"), };
        print(value.first);
        words: [string; 2] = [mark("item1"), mark("item2")];
        print(words[1]);
        print(array()[index()]);
        print(false && mark("skipped1") == mark("skipped2"));
        print(true || mark("skipped3") != mark("skipped4"));
        for (mut flag: bool = true; flag; flag = mark("update1") == mark("update2")) {
            if mark("condition1") != mark("condition2") { continue; }
        }
    "#,
    );
}

#[test]
fn sequencing_also_preserves_numeric_effects_without_strings() {
    let Some(c) = NativeC::new() else { return };
    let source = r#"
        fn mark(value: i64) -> i64 { print(value); return value; }
        fn pair(left: i64, right: i64) -> i64 { return left + right; }
        print(mark(1) + mark(2));
        print(pair(mark(3), mark(4)));
        print(mark(5) < mark(6));
    "#;
    for optimization in ["-O0", "-O2"] {
        let actual = c.run(source, optimization);
        assert!(actual.status.success());
        assert!(actual.stderr.is_empty());
        // 数値のみの既存C出力ではWindowsのテキストモードを維持します。
        assert_eq!(
            String::from_utf8(actual.stdout)
                .unwrap()
                .replace("\r\n", "\n"),
            run_vm(source).unwrap()
        );
    }
    let source = "fn divide() -> i64 { return 1 / 0; } fn index() -> i64 { return [1][2]; } print(divide() + index());";
    assert!(run_vm(source).is_err());
    for optimization in ["-O0", "-O2"] {
        let failed = c.run(source, optimization);
        assert!(!failed.status.success());
        let stderr = String::from_utf8_lossy(&failed.stderr);
        assert!(
            stderr.contains("cannot divide an integer by zero"),
            "{stderr}"
        );
        assert!(!stderr.contains("array index out of bounds"), "{stderr}");
    }
}

#[test]
fn string_bindings_do_not_collide_with_outer_names_or_runtime_helpers() {
    let Some(c) = NativeC::new() else { return };
    c.matches_vm(
        r#"
        text: string = "outer";
        if true {
            text: infer = text;
            print(text);
        }
        for (mut text: string = text; text == "outer"; text = "done") {
            print(text);
        }
        string_equal: string = "helper name";
        print_string: string = "print helper name";
        print(string_equal == "helper name");
        print(print_string);
        print(text);
    "#,
    );
}

#[test]
fn all_current_examples_run_as_generated_c() {
    let Some(c) = NativeC::new() else { return };
    let mut paths: Vec<_> =
        fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "prim")
            })
            .collect();
    paths.sort();
    assert!(!paths.is_empty());
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let expected = run_vm(&source).unwrap();
        for optimization in ["-O0", "-O2"] {
            let actual = c.run(&source, optimization);
            assert!(
                actual.status.success(),
                "{}: {}",
                path.display(),
                String::from_utf8_lossy(&actual.stderr)
            );
            assert!(
                actual.stderr.is_empty(),
                "{}: {}",
                path.display(),
                String::from_utf8_lossy(&actual.stderr)
            );
            let mut output = String::from_utf8(actual.stdout).unwrap();
            // 文字列の2例は別テストでも改行を含めて完全一致を検証します。
            if !path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("string_")
            {
                output = output.replace("\r\n", "\n");
            }
            assert_eq!(output, expected, "{} {optimization}", path.display());
        }
    }
}
