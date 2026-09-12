#[path = "support/crash_dialogs.rs"]
mod crash_dialogs;
#[path = "support/process.rs"]
mod process;

use primer_lang::{
    ast, bytecode, codegen,
    diagnostic::{Diagnostic, render},
    ir, lexer, parser, run_bytecode,
    source::{SourceId, SourceMap, Span},
};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Duration,
};

const LIBRARY: &str = include_str!("../examples/source_files/values.prim");
const MAIN: &str = include_str!("../examples/source_files/main.prim");
const FAILURE: &str = include_str!("../examples/source_files/failure.prim");

fn split(entry: &str) -> (SourceMap, ir::Program) {
    let mut sources = SourceMap::new();
    let library = sources.add("values.prim", LIBRARY);
    let entry = sources.add("entry.prim", entry);
    let mut ast = ast::Program { items: Vec::new() };
    for id in [library, entry] {
        ast.items.extend(
            parser::parse(lexer::lex_source(sources.get(id).unwrap()).unwrap())
                .unwrap()
                .items,
        );
    }
    (sources, ir::builder::build(&ast).unwrap())
}

#[test]
fn file_positions_preserve_bytes_and_do_not_alias_equal_offsets_or_names() {
    let mut sources = SourceMap::new();
    let a = sources.add("same.prim", "// 日本語\r\nprint(\"e\u{301}\0\r\n\");");
    let b = sources.add("same.prim", "// 日本語\r\nprint(\"é\0\r\n\");");
    assert_ne!(a, b);
    assert_eq!(sources.files().count(), 2);
    let offset = sources.get(a).unwrap().text().find("print").unwrap();
    let span = Span::in_source(a, offset, offset + 5);
    assert_ne!(span, span.with_source(b));
    let (file, position) = sources.resolve(span).unwrap();
    assert_eq!(file.id(), a);
    assert_eq!((position.line(), position.column()), (2, 1));
    assert_eq!(sources.slice(span), Some("print"));
    assert!(sources.get(a).unwrap().text().contains("e\u{301}\0\r\n"));
    assert!(sources.get(b).unwrap().text().contains("é\0\r\n"));
    assert!(sources.resolve(Span::in_source(a, 4, 5)).is_none()); // UTF-8の途中
    assert!(sources.resolve(Span::in_source(a, 0, 999)).is_none());
    assert!(sources.resolve(Span::empty(0)).is_none());
    let end = sources.get(a).unwrap().text().len();
    assert!(sources.resolve(Span::in_source(a, end, end)).is_some());
}

#[test]
fn lex_parse_and_semantic_errors_identify_the_right_file() {
    for text in ["print(\"\\q\");", "print(1 + );", "print(unknown);"] {
        let mut sources = SourceMap::new();
        sources.add("decoy.prim", "print(0);");
        let id = sources.add("計算\n\u{1b}.prim", format!("// 日本語\r\n{text}"));
        let error = primer_lang::compile_source(sources.get(id).unwrap()).unwrap_err();
        let span = error.primary_span().unwrap();
        assert_eq!(span.source_id(), id);
        let rendered = render::render_compact_with_sources(&error, &sources);
        assert!(rendered.contains(r"計算\n\u{1b}.prim:2:"), "{rendered}");
        assert!(!rendered.contains("decoy"));
        // 単一ソース用rendererへ別の本文を渡しても、それを位置解決には使いません。
        assert!(render::render_compact(&error, "wrong").contains("file=2 byte"));
        assert!(
            render::render_compact_with_sources(&error, &SourceMap::new()).contains("file=2 byte")
        );
    }
    let mut sources = SourceMap::new();
    let a = sources.add("a", "print(");
    let b = sources.add("b", "1);");
    let mut tokens = lexer::lex_source(sources.get(a).unwrap()).unwrap();
    tokens.pop();
    tokens.extend(lexer::lex_source(sources.get(b).unwrap()).unwrap());
    assert!(
        parser::parse(tokens)
            .unwrap_err()
            .message()
            .contains("different source files")
    );
}

#[test]
fn runtime_origin_points_into_the_function_definition_and_keeps_prior_output() {
    let (sources, ir) = split(FAILURE);
    let error = run_bytecode(&bytecode::lower(&ir).unwrap()).unwrap_err();
    assert_eq!(error.vm_error().output(), "開始\n計算\n5\n計算\n");
    let failure = error.runtime_failure().unwrap();
    assert_eq!(failure.code.name(), "division-by-zero");
    assert_eq!(sources.slice(failure.span), Some("value / divisor"));
    assert_eq!(
        sources.resolve(failure.span).unwrap().0.name(),
        "values.prim"
    );
    assert!(failure.record().contains(" file=1 bytes="));
    assert!(!failure.record().contains("values.prim"));
    let text = primer_lang::vm::render::render_compact_with_sources(
        error.vm_error(),
        &sources,
        failure.span,
    );
    assert!(text.contains("values.prim:13:12"), "{text}");
    let diagnostic = Diagnostic::new("test", failure.span);
    assert!(
        render::render_compact_with_sources(&diagnostic, &sources).contains("values.prim:13:12")
    );
}

#[test]
fn named_ir_and_annotations_keep_file_ids_without_embedding_paths() {
    let mut sources = SourceMap::new();
    let id = sources.add("private/path.prim", "print(1 / 0);");
    let program = primer_lang::compile_source_to_ir(sources.get(id).unwrap()).unwrap();
    assert_eq!(program.statements[0].span.source_id(), id);
    let llvm = codegen::llvm::emit_llvm_with_options(
        &program,
        codegen::llvm::Options {
            target: Some(codegen::llvm::Target::X86_64UnknownLinuxGnu),
            annotate_origins: true,
        },
    )
    .unwrap();
    let asm = codegen::x86_64::emit_asm_with_origins(
        &program,
        codegen::x86_64::Target::X86_64UnknownLinuxGnu,
    )
    .unwrap();
    for text in [llvm, asm] {
        assert!(text.contains("file=1 bytes"));
        assert!(!text.contains("private/path"));
    }
    assert_eq!(Span::new(0, 1).source_id(), SourceId::ANONYMOUS);
}

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        crash_dialogs::suppress();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "primer-source-files-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn run(&self, command: &mut Command, label: &str) -> Output {
        process::bounded_output(command, &self.0, label, Duration::from_secs(30)).unwrap()
    }
    fn success(&self, command: &mut Command, label: &str) {
        let output = self.run(command, label);
        assert!(
            output.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fn tool(&self, variable: &str, default: &str, flag: &str) -> Option<OsString> {
        let configured = std::env::var_os(variable);
        let tool = configured.clone().unwrap_or_else(|| default.into());
        let output = process::bounded_output(
            Command::new(&tool).arg(flag),
            &self.0,
            variable,
            Duration::from_secs(10),
        );
        if output.is_ok_and(|o| o.status.success()) {
            Some(tool)
        } else {
            assert!(
                configured.is_none(),
                "required tool unavailable: {variable}"
            );
            eprintln!("source-file execution skipped: {variable} unavailable");
            None
        }
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check_execution(program: &ir::Program, output: Output, route: &str) {
    let vm = run_bytecode(&bytecode::lower(program).unwrap());
    match vm {
        Ok(expected) => {
            assert!(output.status.success(), "{route}: {:?}", output.status);
            assert!(
                output.stderr.is_empty(),
                "{route}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, expected.as_bytes(), "{route}");
        }
        Err(error) => {
            assert_eq!(
                output.stdout,
                error.vm_error().output().as_bytes(),
                "{route}"
            );
            assert_eq!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .replace("\r\n", "\n"),
                format!("primer: {}\n", error.runtime_failure().unwrap().record()),
                "{route}"
            );
            if route == "wat" {
                assert_eq!(output.status.code(), Some(1));
            } else {
                let illegal = matches!(route, "llvm" | "asm" | "object");
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    assert_eq!(
                        output.status.signal(),
                        Some(if illegal { 4 } else { 6 }),
                        "{route}"
                    );
                }
                #[cfg(windows)]
                {
                    let code = output.status.code().map(|c| c as u32);
                    if illegal {
                        assert_eq!(code, Some(0xc000001d), "{route}");
                    } else {
                        assert!(matches!(code, Some(3 | 0xc0000409)), "{route}: {code:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn split_and_single_file_examples_execute_identically_on_available_routes() {
    let w = Workspace::new();
    let cc = w.tool("PRIMER_TEST_CC", "clang", "--version");
    let llvm = w.tool("PRIMER_TEST_LLVM_CLANG", "clang", "--version");
    let node = w.tool("PRIMER_TEST_NODE", "node", "--version");
    let qbe = if cfg!(target_os = "linux") {
        w.tool("PRIMER_TEST_QBE", "qbe", "-h")
    } else {
        None
    };
    let wat = std::env::var_os("PRIMER_TEST_WAT2WASM_JS");
    if let Some(wat) = &wat {
        assert!(Path::new(wat).is_file());
    } else {
        eprintln!("source-file WAT execution skipped: PRIMER_TEST_WAT2WASM_JS not configured");
    }
    let target = if cfg!(windows) {
        codegen::x86_64::Target::X86_64PcWindowsMsvc
    } else {
        codegen::x86_64::Target::X86_64UnknownLinuxGnu
    };
    let llvm_target = codegen::llvm::Target::parse(target.triple()).unwrap();
    let cases = [
        (
            MAIN,
            "18446744073709551615\n2\n観測\0\r\n\nfalse\n計算\n5\n",
            None,
        ),
        (
            FAILURE,
            "開始\n計算\n5\n計算\n",
            Some(("division-by-zero", "value / divisor", "values.prim")),
        ),
        (
            "print(\"添字\"); mut a: [u64; 1] = [0]; a[1] = divide(10, 0);",
            "添字\n",
            Some(("array-index-out-of-bounds", "[1]", "entry.prim")),
        ),
    ];
    for (case, (entry, expected, failure)) in cases.into_iter().enumerate() {
        let (sources, split) = split(entry);
        let bytecode = bytecode::lower(&split).unwrap();
        let vm = run_bytecode(&bytecode);
        if let Some((code, expression, name)) = failure {
            let error = vm.unwrap_err();
            assert_eq!(error.vm_error().output(), expected);
            let record = error.runtime_failure().unwrap();
            assert_eq!(record.code.name(), code);
            assert_eq!(sources.slice(record.span), Some(expression));
            assert_eq!(sources.resolve(record.span).unwrap().0.name(), name);
        } else {
            assert_eq!(vm.unwrap(), expected);
        }
        let single = primer_lang::compile_to_ir(&format!("{LIBRARY}\n{entry}")).unwrap();
        let single_vm = run_bytecode(&bytecode::lower(&single).unwrap());
        match single_vm {
            Ok(text) => {
                assert!(failure.is_none());
                assert_eq!(text, expected);
            }
            Err(error) => {
                assert_eq!(error.vm_error().output(), expected);
                assert_eq!(
                    error.runtime_failure().unwrap().code.name(),
                    failure.unwrap().0
                );
            }
        }
        for (form, program) in [("split", &split), ("single", &single)] {
            for route in ["c", "llvm", "qbe", "asm", "object", "wat"] {
                let label = format!("{case}/{form}/{route}");
                if route == "wat" {
                    if let (Some(node), Some(wat)) = (&node, &wat) {
                        let input = w.0.join("program.wat");
                        let wasm = w.0.join("program.wasm");
                        fs::write(&input, codegen::emit_wat(program).unwrap()).unwrap();
                        w.success(
                            Command::new(node).arg(wat).arg(&input).arg("-o").arg(&wasm),
                            &format!("{label}/compile"),
                        );
                        let output = w.run(
                            Command::new(node)
                                .arg(
                                    Path::new(env!("CARGO_MANIFEST_DIR"))
                                        .join("tests/support/run_wasm.cjs"),
                                )
                                .arg(wasm),
                            &format!("{label}/run"),
                        );
                        check_execution(program, output, route);
                    }
                    continue;
                }
                let compiler = if route == "llvm" {
                    llvm.as_ref()
                } else {
                    cc.as_ref()
                };
                let Some(compiler) = compiler else {
                    continue;
                };
                if route == "qbe" && qbe.is_none() {
                    continue;
                }
                let input = w.0.join(match route {
                    "c" => "program.c",
                    "llvm" => "program.ll",
                    "object" => {
                        if cfg!(windows) {
                            "program.obj"
                        } else {
                            "program.o"
                        }
                    }
                    _ => "program.s",
                });
                match route {
                    "c" => fs::write(&input, codegen::emit_c(program).unwrap()).unwrap(),
                    "llvm" => fs::write(
                        &input,
                        codegen::llvm::emit_llvm_with_target(program, Some(llvm_target)).unwrap(),
                    )
                    .unwrap(),
                    "asm" => fs::write(
                        &input,
                        codegen::x86_64::emit_asm_with_origins(program, target).unwrap(),
                    )
                    .unwrap(),
                    "object" => fs::write(
                        &input,
                        codegen::x86_64::emit_object(program, target, true).unwrap(),
                    )
                    .unwrap(),
                    "qbe" => {
                        let ssa = w.0.join("program.ssa");
                        fs::write(
                            &ssa,
                            codegen::qbe::emit_qbe_with_target(
                                program,
                                Some(codegen::qbe::Target::X86_64UnknownLinuxGnu),
                            )
                            .unwrap(),
                        )
                        .unwrap();
                        w.success(
                            Command::new(qbe.as_ref().unwrap())
                                .arg("-o")
                                .arg(&input)
                                .arg(ssa),
                            &format!("{label}/lower"),
                        );
                    }
                    _ => unreachable!(),
                }
                for optimization in ["-O0", "-O2"] {
                    let exe = w.0.join(if cfg!(windows) {
                        "program.exe"
                    } else {
                        "program"
                    });
                    let mut command = Command::new(compiler);
                    command.arg(optimization);
                    if route == "llvm" {
                        command.arg(format!("--target={}", target.triple()));
                    }
                    if route == "c" {
                        command.args(["-std=c11", "-pedantic-errors"]);
                    }
                    command.arg(&input).arg("-o").arg(&exe);
                    if !cfg!(windows) {
                        command.arg("-lm");
                    }
                    w.success(&mut command, &format!("{label}/{optimization}/link"));
                    check_execution(
                        program,
                        w.run(
                            &mut Command::new(exe),
                            &format!("{label}/{optimization}/run"),
                        ),
                        route,
                    );
                }
            }
        }
    }
}
