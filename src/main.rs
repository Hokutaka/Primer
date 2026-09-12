use primer_lang::{diagnostic::Diagnostic, modules::Compilation};
use std::{env, fs, path::PathBuf, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("primer: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);

    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };

    match command.as_str() {
        // コードの構文チェック
        "check" => {
            let input = required_path(args.next(), "missing input file")?;

            reject_extra(args)?;

            let source = read_source(&input)?;

            render_compilation_result(primer_lang::semantic::check(&source.program), &source)?;

            println!("OK {}", input.display());

            Ok(())
        }

        // 依存ファイルと正確な本文を明示的に出力します。
        "emit-sources" => {
            let input = required_path(args.next(), "missing input file")?;
            let rest: Vec<String> = args.collect();
            let output =
                parse_output_option(&rest, "primer emit-sources <file> [-o <sources.json>]")?;
            let source = read_source(&input)?;
            ir_for(&source)?;
            write_or_print(output, source.source_manifest())
        }

        // Primer IR 生成
        "emit-ir" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-ir <file> [-o <output.pir>]")?;

            let source = read_source(&input)?;

            let ir = render_compilation_result(
                Ok(primer_lang::ir::text::emit(&ir_for(&source)?)),
                &source,
            )?;

            write_or_print(output, ir)
        }

        // C コード生成
        "emit-c" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-c <file> [-o <output.c>]")?;

            let source = read_source(&input)?;

            let c = render_compilation_result(
                primer_lang::codegen::emit_c(&ir_for(&source)?),
                &source,
            )?;

            write_or_print(output, c)
        }

        // LLVM コード生成
        "emit-llvm" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let (output, target, annotate_origins) =
                parse_native_options(&rest, "emit-llvm", "ll")?;
            let target = match target.as_deref() {
                None => None,
                Some(value) => Some(primer_lang::codegen::llvm::Target::parse(value).ok_or_else(|| {
                    format!("unsupported LLVM target `{value}`; expected x86_64-unknown-linux-gnu or x86_64-pc-windows-msvc")
                })?),
            };

            let source = read_source(&input)?;

            let llvm = render_compilation_result(
                primer_lang::codegen::llvm::emit_llvm_with_options(
                    &ir_for(&source)?,
                    primer_lang::codegen::llvm::Options {
                        target,
                        annotate_origins,
                    },
                ),
                &source,
            )?;

            write_or_print(output, llvm)
        }

        // WAT コード生成
        "emit-wat" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-wat <file> [-o <output.wat>]")?;

            let source = read_source(&input)?;

            let wat = render_compilation_result(
                primer_lang::codegen::emit_wat(&ir_for(&source)?),
                &source,
            )?;

            write_or_print(output, wat)
        }

        // QBE コード生成
        "emit-qbe" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let (output, target, _) = parse_native_options(&rest, "emit-qbe", "ssa")?;
            let target = match target.as_deref() {
                None => None,
                Some(value) => Some(primer_lang::codegen::qbe::Target::parse(value).ok_or_else(
                    || {
                        format!(
                            "unsupported QBE target `{value}`; expected x86_64-unknown-linux-gnu"
                        )
                    },
                )?),
            };

            let source = read_source(&input)?;

            let qbe = render_compilation_result(
                primer_lang::codegen::qbe::emit_qbe_with_target(&ir_for(&source)?, target),
                &source,
            )?;

            write_or_print(output, qbe)
        }

        // Direct Assembly コード生成
        "emit-asm" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let (output, target, annotate_origins) = parse_native_options(&rest, "emit-asm", "s")?;
            let target = match target.as_deref() {
                None => primer_lang::codegen::x86_64::Target::X86_64PcWindowsMsvc,
                Some(value) => primer_lang::codegen::x86_64::Target::parse(value).ok_or_else(|| format!("unsupported assembly target `{value}`; expected x86_64-unknown-linux-gnu or x86_64-pc-windows-msvc"))?,
            };

            let source = read_source(&input)?;

            let asm = render_compilation_result(
                if annotate_origins {
                    primer_lang::codegen::x86_64::emit_asm_with_origins(&ir_for(&source)?, target)
                } else {
                    primer_lang::codegen::x86_64::emit_asm(&ir_for(&source)?, target)
                },
                &source,
            )?;

            write_or_print(output, asm)
        }

        // バイナリは出力先とターゲットを必須にし、端末へ暗黙に書き出しません。
        "emit-obj" => {
            let input = required_path(args.next(), "missing input file")?;
            let rest: Vec<String> = args.collect();
            let (output, target, origins) = parse_native_options(&rest, "emit-obj", "o")?;
            let output = output.ok_or("emit-obj requires -o <output.o>")?;
            let target = target.ok_or("emit-obj requires an explicit --target")?;
            let target = primer_lang::codegen::x86_64::Target::parse(&target)
                .ok_or("unsupported native object target")?;
            let source = read_source(&input)?;
            let bytes = render_compilation_result(
                primer_lang::codegen::x86_64::emit_object(&ir_for(&source)?, target, origins),
                &source,
            )?;
            fs::write(&output, bytes)
                .map_err(|e| format!("failed to write {}: {e}", output.display()))
        }

        // Primer Bytecode 生成
        "emit-bytecode" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output =
                parse_output_option(&rest, "primer emit-bytecode <file> [-o <output.pbc>]")?;

            let source = read_source(&input)?;

            let bytecode = render_compilation_result(
                primer_lang::bytecode::lower(&ir_for(&source)?)
                    .map(|program| primer_lang::bytecode::format_program(&program)),
                &source,
            )?;

            write_or_print(output, bytecode)
        }

        // Primer VM 実行
        "run" => {
            let input = required_path(args.next(), "missing input file")?;
            let rest: Vec<_> = args.collect();
            let runtime_format = match rest.as_slice() {
                [] => false,
                [flag, format] if flag == "--diagnostic-format" && format == "runtime-v1" => true,
                _ => return Err("usage: primer run <file> [--diagnostic-format runtime-v1]".into()),
            };

            let source = read_source(&input)?;

            let bytecode = render_compilation_result(
                primer_lang::bytecode::lower(&ir_for(&source)?),
                &source,
            )?;
            let output = primer_lang::run_bytecode(&bytecode).map_err(|error| {
                print!("{}", error.vm_error().output());
                if runtime_format && let Some(failure) = error.runtime_failure() {
                    return failure.record();
                }
                source.render_execution(&error)
            })?;

            print!("{output}");

            Ok(())
        }

        "--version" | "-V" | "version" => {
            println!("primer {}", env!("CARGO_PKG_VERSION"));

            Ok(())
        }

        "--help" | "-h" | "help" => {
            print_help();
            Ok(())
        }

        other => Err(format!("unknown command `{other}`")),
    }
}

fn render_compilation_result<T>(
    result: Result<T, Diagnostic>,
    source: &Compilation,
) -> Result<T, String> {
    result.map_err(|diagnostic| source.render(&diagnostic))
}

fn ir_for(source: &Compilation) -> Result<primer_lang::ir::Program, String> {
    render_compilation_result(source.to_ir(), source)
}

fn required_path(value: Option<String>, message: &str) -> Result<PathBuf, String> {
    value.map(PathBuf::from).ok_or_else(|| message.to_owned())
}

fn read_source(path: &std::path::Path) -> Result<Compilation, String> {
    primer_lang::modules::load(path).map_err(|error| error.render())
}

fn reject_extra(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    if let Some(extra) = args.next() {
        Err(format!("unexpected argument `{extra}`"))
    } else {
        Ok(())
    }
}

fn parse_output_option(args: &[String], usage: &str) -> Result<Option<PathBuf>, String> {
    match args {
        [] => Ok(None),

        [flag, path] if flag == "-o" || flag == "--output" => Ok(Some(PathBuf::from(path))),

        _ => Err(format!("usage: {usage}")),
    }
}

fn parse_native_options(
    args: &[String],
    route: &str,
    extension: &str,
) -> Result<(Option<PathBuf>, Option<String>, bool), String> {
    let mut output = None;
    let mut target = None;
    let mut annotate_origins = false;
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        if flag == "--annotate-origins"
            && matches!(route, "emit-llvm" | "emit-asm" | "emit-obj")
            && !annotate_origins
        {
            annotate_origins = true;
            continue;
        }
        match (flag.as_str(), args.next()) {
            ("-o" | "--output", Some(path)) if output.is_none() => {
                output = Some(PathBuf::from(path));
            }
            ("--target", Some(value)) if target.is_none() => {
                target = Some(value.clone());
            }
            _ => {
                return Err(format!(
                    "usage: primer {route} <file> [--target <triple>] [-o <output.{extension}>]"
                ));
            }
        }
    }
    Ok((output, target, annotate_origins))
}

fn write_or_print(output: Option<PathBuf>, content: String) -> Result<(), String> {
    match output {
        Some(path) => fs::write(&path, content)
            .map_err(|e| format!("failed to write {}: {e}", path.display())),

        None => {
            print!("{content}");
            Ok(())
        }
    }
}

fn print_help() {
    println!(
        "Primer {}\n\n\
         A small experimental language with observable code generation.\n\n\
         USAGE:\n\
           primer check <file>\n\
           primer emit-sources <file> [-o <sources.json>]\n\
           primer emit-ir <file> [-o <output.pir>]\n\
           primer emit-c <file> [-o <output.c>]\n\
           primer emit-llvm <file> [--target <triple>] [--annotate-origins] [-o <output.ll>]\n\
           primer emit-wat <file> [-o <output.wat>]\n\
           primer emit-qbe <file> [--target <triple>] [-o <output.ssa>]\n\
           primer emit-asm <file> [--target <triple>] [--annotate-origins] [-o <output.s>]\n\
           primer emit-obj <file> --target <triple> [--annotate-origins] -o <output.o>\n\
           primer emit-bytecode <file> [-o <output.pbc>]\n\
           primer run <file> [--diagnostic-format runtime-v1]\n\
           primer --version\n",
        env!("CARGO_PKG_VERSION")
    );
}
