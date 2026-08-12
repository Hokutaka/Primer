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

            primer_lang::compile(&source)?;

            println!("OK {}", input.display());

            Ok(())
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
           primer emit-c <file> [-o <output.c>]\n\
           primer emit-llvm <file> [-o <output.ll>]\n\
           primer --version\n",
        env!("CARGO_PKG_VERSION")
    );
}
