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

            primer_lang::compile(&source).map_err(|diagnostic| {
                primer_lang::diagnostic::render::render_compact(&diagnostic, &source)
            })?;

            println!("OK {}", input.display());

            Ok(())
        }

        // Primer IR 生成
        "emit-ir" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-ir <file> [-o <output.pir>]")?;

            let source = read_source(&input)?;

            let ir = primer_lang::compile_to_ir_text(&source)?;

            write_or_print(output, ir)
        }

        // C コード生成
        "emit-c" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-c <file> [-o <output.c>]")?;

            let source = read_source(&input)?;

            let c = primer_lang::compile_to_c(&source)?;

            write_or_print(output, c)
        }

        // LLVM コード生成
        "emit-llvm" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-llvm <file> [-o <output.ll>]")?;

            let source = read_source(&input)?;

            let llvm = primer_lang::compile_to_llvm(&source)?;

            write_or_print(output, llvm)
        }

        // WAT コード生成
        "emit-wat" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-wat <file> [-o <output.wat>]")?;

            let source = read_source(&input)?;

            let wat = primer_lang::compile_to_wat(&source)?;

            write_or_print(output, wat)
        }

        // QBE コード生成
        "emit-qbe" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-qbe <file> [-o <output.ssa>]")?;

            let source = read_source(&input)?;

            let qbe = primer_lang::compile_to_qbe(&source)?;

            write_or_print(output, qbe)
        }

        // Direct Assembly コード生成
        "emit-asm" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output = parse_output_option(&rest, "primer emit-asm <file> [-o <output.s>]")?;

            let source = read_source(&input)?;

            let asm = primer_lang::compile_to_x86_64_win_asm(&source)?;

            write_or_print(output, asm)
        }

        // Primer Bytecode 生成
        "emit-bytecode" => {
            let input = required_path(args.next(), "missing input file")?;

            let rest: Vec<String> = args.collect();

            let output =
                parse_output_option(&rest, "primer emit-bytecode <file> [-o <output.pbc>]")?;

            let source = read_source(&input)?;

            let bytecode = primer_lang::compile_to_bytecode_text(&source)?;

            write_or_print(output, bytecode)
        }

        // Primer VM 実行
        "run" => {
            let input = required_path(args.next(), "missing input file")?;

            reject_extra(args)?;

            let source = read_source(&input)?;

            let output = primer_lang::run_vm(&source)?;

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

fn required_path(value: Option<String>, message: &str) -> Result<PathBuf, String> {
    value.map(PathBuf::from).ok_or_else(|| message.to_owned())
}

fn read_source(path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))
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
           primer emit-ir <file> [-o <output.pir>]\n\
           primer emit-c <file> [-o <output.c>]\n\
           primer emit-llvm <file> [-o <output.ll>]\n\
           primer emit-wat <file> [-o <output.wat>]\n\
           primer emit-qbe <file> [-o <output.ssa>]\n\
           primer emit-asm <file> [-o <output.s>]\n\
           primer emit-bytecode <file> [-o <output.pbc>]\n\
           primer run <file>\n\
           primer --version\n",
        env!("CARGO_PKG_VERSION")
    );
}
