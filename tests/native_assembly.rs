#[path = "support/crash_dialogs.rs"]
mod crash_dialogs;
#[path = "support/string_cases.rs"]
mod string_cases;
#[path = "support/termination.rs"]
mod termination;
#[path = "support/u64_cases.rs"]
mod u64_cases;

use primer_lang::{
    codegen::x86_64::Target, compile_to_asm_with_target, compile_to_x86_64_win_asm, run_vm,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

// pipeのEOFを待たず、子プロセスの終了と取得済み出力を別々に観測します。
// 異常停止を大量に実行するCIでも、どのケースを待っているか残します。
fn bounded_output(
    command: &mut Command,
    directory: &Path,
    label: &str,
    limit: Duration,
) -> Result<Output, String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let stdout = directory.join(format!("process-{id}.stdout"));
    let stderr = directory.join(format!("process-{id}.stderr"));
    command.stdout(Stdio::from(fs::File::create(&stdout).unwrap()));
    command.stderr(Stdio::from(fs::File::create(&stderr).unwrap()));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let trace = std::env::var_os("PRIMER_TEST_TRACE").is_some();
    if trace {
        use std::io::Write;
        let _ = writeln!(std::io::stderr().lock(), "[native-process] start {label}");
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("{label}: {error}"))?;
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("{label}: {error}"))?
        {
            break status;
        }
        if start.elapsed() >= limit {
            #[cfg(unix)]
            {
                // この呼び出し専用のprocess groupだけを終了します。
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                unsafe {
                    kill(-(child.id() as i32), 9);
                }
            }
            #[cfg(windows)]
            {
                // テストが起動した子とその子孫だけが対象です。
                let _ = Command::new("taskkill")
                    .args(["/PID", &child.id().to_string(), "/T", "/F"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{label}: timed out after {limit:?}; stdout={:?}; stderr={:?}",
                String::from_utf8_lossy(&fs::read(&stdout).unwrap()),
                String::from_utf8_lossy(&fs::read(&stderr).unwrap())
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if trace {
        use std::io::Write;
        let _ = writeln!(
            std::io::stderr().lock(),
            "[native-process] end {label}: {status} in {:?}",
            start.elapsed()
        );
    }
    Ok(Output {
        status,
        stdout: fs::read(stdout).unwrap(),
        stderr: fs::read(stderr).unwrap(),
    })
}

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        crash_dialogs::suppress();
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Windowsの時計の分解能だけに依存せず、同時実行するテストを分離します。
        loop {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("primer-native-{}-{stamp}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create test workspace: {error}"),
            }
        }
    }
}

#[test]
fn concurrent_test_workspaces_are_independent() {
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(16));
    let handles: Vec<_> = (0..16)
        .map(|index| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let workspace = Workspace::new();
                fs::write(workspace.0.join("marker"), index.to_string()).unwrap();
                (workspace, index)
            })
        })
        .collect();
    let workspaces: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let paths: std::collections::BTreeSet<_> =
        workspaces.iter().map(|(w, _)| w.0.clone()).collect();
    assert_eq!(paths.len(), 16);
    for (workspace, index) in &workspaces {
        assert_eq!(
            fs::read_to_string(workspace.0.join("marker")).unwrap(),
            index.to_string()
        );
    }
}

#[test]
fn native_process_deadline_reports_output_and_stops_an_infinite_loop() {
    let workspace = Workspace::new();
    let node = std::env::var_os("PRIMER_TEST_NODE").unwrap_or_else(|| "node".into());
    let result = bounded_output(Command::new(node).args(["-e", "console.log('started'); setInterval(() => {}, 1000); process.on('SIGTERM', () => {}); "]),
        &workspace.0, "intentional-hang", Duration::from_secs(1));
    let error = result.unwrap_err();
    assert!(
        error.contains("intentional-hang: timed out") && error.contains("started"),
        "{error}"
    );
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cases() -> Vec<(String, String)> {
    let mut cases: Vec<_> = string_cases::CASES
        .iter()
        .chain(
            [
                u64_cases::EXAMPLE,
                u64_cases::BOUNDARIES,
                string_cases::BYTE_LENGTH,
            ]
            .iter(),
        )
        .map(|&(source, expected)| (source.to_owned(), expected.to_owned()))
        .collect();
    cases.push((string_cases::UNUSED_DEFAULT.into(), "1\ntrue\n".into()));
    let compact = "fn next() -> i32 { print(7); return 3; } print(f64(next())); print(f64(1 / 2)); print(f64(1) / f64(2));";
    cases.push((compact.into(), "7\n3\n0\n0.5\n".into()));
    cases.push((
        compact.replace("f64(", "convert<f64>("),
        "7\n3\n0\n0.5\n".into(),
    ));
    let values = (0..600)
        .map(|index| {
            if index == 599 {
                "18446744073709551615"
            } else {
                "0"
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    cases.push((format!("print(\"large frame\"); fn take(values: [u64; 600], x: f64, y: u64, z: f32) -> u64 {{ print(x + f64(z)); return values[599] + y; }} data: [u64; 600] = [{values}]; print(take(data, 2.5, 0, 1.5));"), "large frame\n4\n18446744073709551615\n".into()));
    cases.push((format!("print(\"large return\"); fn copy(values: [u64; 600]) -> [u64; 600] {{ return values; }} data: [u64; 600] = [{values}]; mut copied: [u64; 600] = copy(data); copied[599] = 7; print(data[599]); print(copied[599]);"), "large return\n18446744073709551615\n7\n".into()));
    cases.push((include_str!("../examples/native_values.prim").into(), "native values\n18446744073709551615\n-7\n4\n4\n18446744073709551615\n9223372036854775808\n日本語\0\n".into()));
    let mut paths: Vec<_> =
        fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|s| s == "prim"))
            .collect();
    paths.sort();
    for path in paths {
        let source = fs::read_to_string(path).unwrap();
        let expected = run_vm(&source).unwrap();
        cases.push((source, expected));
    }
    cases
}

#[test]
fn runtime_record_parser_rejects_incomplete_or_unrelated_failures() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/observe-native.cjs");
    let node = std::env::var_os("PRIMER_TEST_NODE").unwrap_or_else(|| "node".into());
    let result = Command::new(node).arg("-e").arg(r#"
const assert = require('node:assert/strict');
const {parseRuntimeFailure: parse} = require(process.argv[1]);
const valid = 'primer: runtime-v1 code=division-by-zero node=2 bytes=6..11\n';
assert.deepEqual(parse(Buffer.from(valid)), {schema:'runtime-v1', code:'division-by-zero', node:2, start:6, end:11});
assert.deepEqual(parse(Buffer.from(valid.replace('\n','\r\n'))), parse(Buffer.from(valid)));
for (const text of ['', 'segmentation fault\n', valid + '\n', valid + valid, valid.trimEnd(),
    valid.replace('division-by-zero','unknown'), valid.replace('6..11','11..6'),
    valid.replace('6..11','6..6'), valid.replace('node=2','node=02'),
    valid.replace('node=2','node=9007199254740992')]) assert.equal(parse(Buffer.from(text)), null, text);
"#).arg(script).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn runtime_failures_match_vm_codes_origins_and_prior_output() {
    use primer_lang::{RunError, compile_to_native_object};
    let cases = [
        ("print(9223372036854775807 + 1);", "integer-overflow"),
        ("print(-(-9223372036854775808));", "integer-overflow"),
        ("print(127i8 + 1);", "integer-overflow"),
        ("print(0u8 - 1);", "integer-overflow"),
        ("print(18446744073709551615u64 + 1);", "integer-overflow"),
        ("print(0u64 - 1);", "integer-overflow"),
        ("print(18446744073709551615u64 * 2);", "integer-overflow"),
        ("print(1 / 0);", "division-by-zero"),
        ("print(1u64 / 0);", "division-by-zero"),
        ("print(-9223372036854775808 / -1);", "division-overflow"),
        ("print(-128i8 / -1);", "division-overflow"),
        ("print(1 % 0);", "remainder-by-zero"),
        ("print(1u64 % 0);", "remainder-by-zero"),
        ("print(1 << -1);", "invalid-shift-count"),
        ("print(0u8 >> 8);", "invalid-shift-count"),
        ("print(0u64 << 64);", "invalid-shift-count"),
        (
            "print(0u64 >> 18446744073709551615u64);",
            "invalid-shift-count",
        ),
        ("print(64i8 << 1);", "integer-overflow"),
        ("print(9223372036854775808u64 << 1);", "integer-overflow"),
        ("print(i8(128));", "integer-conversion-out-of-range"),
        ("print(u64(-1));", "integer-conversion-out-of-range"),
        (
            "print(i64(9223372036854775808u64));",
            "integer-conversion-out-of-range",
        ),
        ("print(f64(9223372036854775807));", "conversion-inexact"),
        ("print(f32(16777217));", "conversion-inexact"),
        ("print(f64(18446744073709551615u64));", "conversion-inexact"),
        ("print(f32(18446744073709551615u64));", "conversion-inexact"),
        ("print(i64(1.5));", "conversion-inexact"),
        ("print(u64(1.5));", "conversion-inexact"),
        ("print(i8(128.0));", "conversion-out-of-range"),
        ("print(u64(-1.0));", "conversion-out-of-range"),
        (
            "print(u64(18446744073709551616.0));",
            "conversion-out-of-range",
        ),
        ("print(i64(0.0 / 0.0));", "conversion-not-finite"),
        ("print(i64(1.0 / 0.0));", "conversion-not-finite"),
        ("print(i64(-1.0 / 0.0));", "conversion-not-finite"),
        ("print(u64(0.0 / 0.0));", "conversion-not-finite"),
        ("print(u64(1.0 / 0.0));", "conversion-not-finite"),
        ("print(u64(-1.0 / 0.0));", "conversion-not-finite"),
        ("print(i64(-0.0));", "conversion-negative-zero"),
        ("print(u64(-0.0));", "conversion-negative-zero"),
        ("print(f32(0.0 / 0.0));", "conversion-nan"),
        ("x: f32 = 0.0 / 0.0; print(f64(x));", "conversion-nan"),
        ("print(f32(0.1));", "conversion-inexact"),
        // f32へ丸めると最大有限値になる場合でも、元の値は範囲外です。
        (
            "print(f32(3.402823466385289e38));",
            "conversion-out-of-range",
        ),
        (
            "print(f32(-3.402823466385289e38));",
            "conversion-out-of-range",
        ),
        (
            "a: [i64; 1] = [1]; print(a[1]);",
            "array-index-out-of-bounds",
        ),
        (
            "a: [[i64; 1]; 1] = [[1]]; print(a[-1][0]);",
            "array-index-out-of-bounds",
        ),
        (
            "fn value() -> i64 { print(999); return 1; } mut a: [[i64; 1]; 1] = [[1]]; a[0][1] = value();",
            "array-index-out-of-bounds",
        ),
        (
            "fn fail() -> i64 { return 1 / 0; } fn outer() -> i64 { return fail(); } print(outer());",
            "division-by-zero",
        ),
        (
            "type P { marker: bool, value: i64 = 1 / 0, } p: P = P { marker: true, };",
            "division-by-zero",
        ),
    ];
    let workspace = Workspace::new();
    let target = if cfg!(windows) {
        Target::X86_64PcWindowsMsvc
    } else {
        Target::X86_64UnknownLinuxGnu
    };
    let cc = std::env::var_os(if cfg!(windows) {
        "PRIMER_TEST_ASM_CLANG"
    } else {
        "PRIMER_TEST_CC"
    })
    .unwrap_or_else(|| if cfg!(windows) { "clang" } else { "cc" }.into());
    let exe = workspace.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    for (case_index, (body, expected_code)) in cases.into_iter().enumerate() {
        // Unicode・CRLFのバイト位置、停止前の出力、短絡で失敗を避ける経路を併せて検証します。
        let source = format!(
            "// 日本語\r\nprint(\"開始\\0\\r\\n\");\r\nprint(false && (1 / 0 == 0));\r\n{body}"
        );
        let RunError::Execution(error) = run_vm(&source).unwrap_err() else {
            panic!("{body}");
        };
        let failure = error.runtime_failure().unwrap();
        assert_eq!(failure.code.name(), expected_code, "{body}");
        assert_eq!(error.vm_error().output(), "開始\0\r\n\nfalse\n", "{body}");
        for encoder in ["asm", "primer"] {
            let input = workspace.0.join(if encoder == "asm" {
                "program.s"
            } else {
                "program.o"
            });
            if encoder == "asm" {
                fs::write(&input, compile_to_asm_with_target(&source, target).unwrap()).unwrap();
            } else {
                fs::write(
                    &input,
                    compile_to_native_object(&source, target, false).unwrap(),
                )
                .unwrap();
            }
            let label = format!("failure-{case_index}/{encoder}/{expected_code}");
            let built = bounded_output(
                Command::new(&cc).arg(&input).arg("-o").arg(&exe),
                &workspace.0,
                &format!("{label}/link"),
                Duration::from_secs(30),
            )
            .unwrap();
            assert!(
                built.status.success(),
                "{body}: {}",
                String::from_utf8_lossy(&built.stderr)
            );
            let result = bounded_output(
                Command::new(&exe).current_dir(&workspace.0),
                &workspace.0,
                &format!("{label}/run"),
                Duration::from_secs(10),
            )
            .unwrap();
            #[cfg(windows)]
            assert_eq!(
                result.status.code().map(|c| c as u32),
                Some(0xc000001d),
                "{body}"
            );
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                assert_eq!(result.status.signal(), Some(4), "{body}");
            }
            assert_eq!(
                result.stdout,
                error.vm_error().output().as_bytes(),
                "{encoder}: {body}"
            );
            assert_eq!(
                String::from_utf8(result.stderr)
                    .unwrap()
                    .replace("\r\n", "\n"),
                format!("primer: {}\n", failure.record()),
                "{encoder}: {body}"
            );
        }
    }
}

#[test]
fn explicit_targets_preserve_windows_output_and_determinism() {
    for (source, expected) in cases() {
        assert_eq!(run_vm(&source).unwrap(), expected);
        let linux = compile_to_asm_with_target(&source, Target::X86_64UnknownLinuxGnu).unwrap();
        assert_eq!(
            linux,
            compile_to_asm_with_target(&source, Target::X86_64UnknownLinuxGnu).unwrap()
        );
        assert!(linux.contains(".section .rodata") && linux.contains(".note.GNU-stack"));
        assert!(!linux.contains("_setmode") && !linux.contains("__chkstk"));
        assert_eq!(
            compile_to_x86_64_win_asm(&source).unwrap(),
            compile_to_asm_with_target(&source, Target::X86_64PcWindowsMsvc).unwrap()
        );
        for target in [Target::X86_64PcWindowsMsvc, Target::X86_64UnknownLinuxGnu] {
            let annotated = primer_lang::compile_to_asm_with_origins(&source, target).unwrap();
            let plain = annotated
                .lines()
                .filter(|line| {
                    !line.starts_with("# primer-") && !line.starts_with("primer_origin_")
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            assert_eq!(plain, compile_to_asm_with_target(&source, target).unwrap());
        }
    }
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
fn linux_assembly_executes_all_examples_and_rejects_invalid_operations() {
    let cc = std::env::var_os("PRIMER_TEST_CC").unwrap_or_else(|| "cc".into());
    let workspace = Workspace::new();
    let input = workspace.0.join("program.s");
    let exe = workspace.0.join("program");
    let run = |source: &str| {
        fs::write(
            &input,
            compile_to_asm_with_target(source, Target::X86_64UnknownLinuxGnu).unwrap(),
        )
        .unwrap();
        let built = Command::new(&cc)
            .arg(&input)
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{source}\n{}",
            String::from_utf8_lossy(&built.stderr)
        );
        Command::new(&exe)
            .current_dir(&workspace.0)
            .output()
            .unwrap()
    };
    for (source, expected) in cases() {
        let actual = run(&source);
        assert!(actual.status.success(), "{source}: {}", actual.status);
        assert!(actual.stderr.is_empty(), "{source}: {:?}", actual.stderr);
        assert_eq!(actual.stdout, expected.as_bytes(), "{source}");
    }
    for source in string_cases::OUT_OF_BOUNDS
        .iter()
        .chain(u64_cases::FAILURES)
    {
        assert!(run_vm(source).is_err());
        termination::assert_expected(
            &run(source),
            termination::Expected::IllegalInstruction,
            source,
        );
    }
}

#[test]
fn machine_artifacts_execute_examples_and_retain_origin_symbols() {
    let node = std::env::var_os("PRIMER_TEST_NODE").unwrap_or_else(|| "node".into());
    let cc = std::env::var_os(if cfg!(windows) {
        "PRIMER_TEST_ASM_CLANG"
    } else {
        "PRIMER_TEST_CC"
    })
    .unwrap_or_else(|| if cfg!(windows) { "clang" } else { "cc" }.into());
    let objdump = std::env::var_os("PRIMER_TEST_OBJDUMP").unwrap_or_else(|| {
        if cfg!(windows) {
            "llvm-objdump"
        } else {
            "objdump"
        }
        .into()
    });
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/observe-native.cjs");
    let workspace = Workspace::new();
    let source_path = workspace.0.join("source.prim");
    let target = if cfg!(windows) {
        Target::X86_64PcWindowsMsvc
    } else {
        Target::X86_64UnknownLinuxGnu
    };
    let invoke = |source: &str, name: &str, failure: bool, encoder: &str| {
        fs::write(&source_path, source).unwrap();
        let directory = workspace.0.join(name);
        let mut command = Command::new(&node);
        command
            .arg(&script)
            .arg("--source")
            .arg(&source_path)
            .args([
                "--target",
                target.triple(),
                "--primer",
                env!("CARGO_BIN_EXE_primer"),
            ])
            .arg("--cc")
            .arg(&cc)
            .arg("--objdump")
            .arg(&objdump)
            .arg("--output-dir")
            .arg(&directory)
            .args(["--encoder", encoder])
            .arg("--run");
        if failure {
            command.arg("--expect-trap");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{source}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let manifest = fs::read_to_string(directory.join("manifest.json")).unwrap();
        assert!(manifest.contains(if failure {
            "expected-failure-confirmed"
        } else {
            "output-matched"
        }));
        let object = fs::read_to_string(directory.join("object.txt")).unwrap();
        assert!(object.contains("primer_origin_n") && object.contains(".text"));
        directory
    };
    for (index, (source, expected)) in cases().into_iter().enumerate() {
        let directory = invoke(&source, &format!("success-{index}"), false, "external");
        let own = invoke(&source, &format!("encoded-{index}"), false, "primer");
        assert_eq!(
            fs::read(own.join("native.stdout")).unwrap(),
            fs::read(directory.join("native.stdout")).unwrap()
        );
        let own_manifest = fs::read_to_string(own.join("manifest.json")).unwrap();
        assert!(own_manifest.contains("encode-object") && !own_manifest.contains("\"assemble\""));
        assert_eq!(
            fs::read(directory.join("vm.stdout")).unwrap(),
            expected.as_bytes()
        );
        if index == 0 {
            let plain_source = directory.join("plain.s");
            let plain_object = directory.join("plain.o");
            fs::write(
                &plain_source,
                compile_to_asm_with_target(&source, target).unwrap(),
            )
            .unwrap();
            let flags = if cfg!(windows) {
                vec![format!("--target={}", target.triple())]
            } else {
                vec!["-m64".into()]
            };
            let assembled = Command::new(&cc)
                .args(flags)
                .arg("-c")
                .arg(&plain_source)
                .arg("-o")
                .arg(&plain_object)
                .output()
                .unwrap();
            assert!(assembled.status.success(), "{:?}", assembled.stderr);
            let bytes = Command::new(&objdump)
                .args(["-s", "-j", ".text"])
                .arg(&plain_object)
                .output()
                .unwrap();
            assert!(bytes.status.success());
            let rows = |text: &str| {
                text.lines()
                    .filter(|line| {
                        line.split_whitespace().next().is_some_and(|word| {
                            word.len() >= 4 && word.chars().all(|c| c.is_ascii_hexdigit())
                        })
                    })
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            };
            let plain = rows(&String::from_utf8(bytes.stdout).unwrap());
            assert!(!plain.is_empty());
            assert_eq!(
                plain,
                rows(&fs::read_to_string(directory.join("text.txt")).unwrap()),
                "origin labels must not change instruction bytes"
            );
        }
    }
    for (index, source) in u64_cases::FAILURES
        .iter()
        .chain(string_cases::OUT_OF_BOUNDS)
        .chain(
            [
                "print(7); print(1 / 0);",
                include_str!("../examples/runtime_failures/overflow.prim"),
                include_str!("../examples/runtime_failures/array_update.prim"),
                include_str!("../examples/runtime_failures/function_division.prim"),
            ]
            .iter(),
        )
        .enumerate()
    {
        invoke(source, &format!("failure-{index}"), true, "external");
        invoke(source, &format!("encoded-failure-{index}"), true, "primer");
    }
}

#[test]
fn object_cli_requires_explicit_target_and_output_and_never_runs_an_assembler() {
    let workspace = Workspace::new();
    let input = workspace.0.join("source.prim");
    let output = workspace.0.join("program.o");
    let source = include_str!("../examples/native_values.prim");
    fs::write(&input, source).unwrap();
    for target in [Target::X86_64PcWindowsMsvc, Target::X86_64UnknownLinuxGnu] {
        let result = Command::new(env!("CARGO_BIN_EXE_primer"))
            .arg("emit-obj")
            .arg(&input)
            .args(["--target", target.triple(), "--annotate-origins", "-o"])
            .arg(&output)
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(result.status.success(), "{:?}", result.stderr);
        assert!(result.stdout.is_empty() && result.stderr.is_empty());
        assert_eq!(
            fs::read(&output).unwrap(),
            primer_lang::compile_to_native_object(source, target, true).unwrap()
        );
    }
    for options in [
        vec![],
        vec!["--target", "unknown"],
        vec![
            "--target",
            "x86_64-unknown-linux-gnu",
            "--annotate-origins",
            "--annotate-origins",
        ],
    ] {
        fs::write(&output, "existing output").unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_primer"))
            .arg("emit-obj")
            .arg(&input)
            .args(options)
            .arg("-o")
            .arg(&output)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert_eq!(fs::read_to_string(&output).unwrap(), "existing output");
    }
    let missing_output = Command::new(env!("CARGO_BIN_EXE_primer"))
        .arg("emit-obj")
        .arg(&input)
        .args(["--target", "x86_64-unknown-linux-gnu"])
        .output()
        .unwrap();
    assert!(!missing_output.status.success() && missing_output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing_output.stderr).contains("requires -o"));
}

#[test]
fn native_observation_rejects_missing_tools_and_existing_output() {
    let node = std::env::var_os("PRIMER_TEST_NODE").unwrap_or_else(|| "node".into());
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/observe-native.cjs");
    let workspace = Workspace::new();
    let source = workspace.0.join("source.prim");
    let output = workspace.0.join("output");
    fs::write(&source, "print(1);").unwrap();
    let invoke = || {
        Command::new(&node)
            .arg(&script)
            .arg("--source")
            .arg(&source)
            .args([
                "--target",
                "x86_64-unknown-linux-gnu",
                "--primer",
                env!("CARGO_BIN_EXE_primer"),
                "--cc",
                "primer-nonexistent-tool",
                "--objdump",
                "primer-nonexistent-tool",
            ])
            .arg("--output-dir")
            .arg(&output)
            .output()
            .unwrap()
    };
    let missing = invoke();
    assert!(!missing.status.success());
    assert!(
        !output.exists(),
        "tool validation should precede output creation"
    );
    fs::create_dir(&output).unwrap();
    fs::write(output.join("keep"), "unchanged").unwrap();
    let existing = invoke();
    assert!(!existing.status.success());
    assert!(String::from_utf8_lossy(&existing.stderr).contains("already exists"));
    assert_eq!(
        fs::read_to_string(output.join("keep")).unwrap(),
        "unchanged"
    );
}

#[test]
fn cli_rejects_unknown_targets_before_replacing_output() {
    let workspace = Workspace::new();
    let input = workspace.0.join("source.prim");
    let output = workspace.0.join("program.s");
    fs::write(&input, "print(1u64);").unwrap();
    for options in [
        vec!["--target", "unknown"],
        vec!["--target"],
        vec!["--annotate-origins", "--annotate-origins"],
    ] {
        fs::write(&output, "existing output").unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_primer"))
            .arg("emit-asm")
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .args(options)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert_eq!(fs::read_to_string(&output).unwrap(), "existing output");
    }
}
