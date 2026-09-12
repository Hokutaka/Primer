#[path = "support/process.rs"]
mod process;
use primer_lang::{bytecode, modules, run_bytecode};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Duration,
};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("primer-modules-{}-{stamp}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn put(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }
    fn cli(&self, command: &str, entry: &Path, args: &[&str]) -> Output {
        process::bounded_output(
            Command::new(env!("CARGO_BIN_EXE_primer"))
                .arg(command)
                .arg(entry)
                .args(args)
                .current_dir(&self.0),
            &self.0,
            command,
            Duration::from_secs(30),
        )
        .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn diamond_imports_share_nominal_types_and_do_not_execute_imported_main() {
    let w = Workspace::new();
    w.put("lib/value.prim", "pub type Value { n: u64, } pub fn main() -> u64 { print(999); return 9; } pub fn make(n: u64) -> Value { return Value { n: n }; }");
    w.put(
        "left.prim",
        "import \"lib/value.prim\" as v; pub fn make() -> v::Value { return v::make(7); }",
    );
    w.put(
        "nested/right.prim",
        "import \"../lib/./value.prim\" as v; pub fn read(x: v::Value) -> u64 { return x.n; }",
    );
    let entry = w.put("main.prim", "import \"left.prim\" as l; import \"nested/right.prim\" as r; fn main() -> void { print(r::read(l::make())); }");
    let compilation = modules::load(&entry).unwrap();
    assert_eq!(compilation.sources.files().count(), 4);
    let ir = compilation.to_ir().unwrap();
    assert_eq!(run_bytecode(&bytecode::lower(&ir).unwrap()).unwrap(), "7\n");
    let output = w.cli("run", &entry, &[]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"7\n");
    assert!(output.stderr.is_empty());
    assert_eq!(
        compilation.source_manifest(),
        modules::load(&entry).unwrap().source_manifest()
    );
}

#[test]
fn same_names_are_local_and_aliases_can_name_the_same_file() {
    let w = Workspace::new();
    w.put(
        "a.prim",
        "pub fn value() -> i64 { return 1; } fn helper() -> i64 { return 3; }",
    );
    w.put(
        "b.prim",
        "pub fn value() -> i64 { return helper(); } fn helper() -> i64 { return 2; }",
    );
    let entry = w.put("main.prim", "import \"a.prim\" as a; import \"b.prim\" as b; import \"./a.prim\" as again; fn value() -> i64 { return 4; } print(a::value()); print(b::value()); print(again::value()); print(value());");
    let compilation = modules::load(&entry).unwrap();
    assert_eq!(compilation.sources.files().count(), 3);
    assert_eq!(
        run_bytecode(&bytecode::lower(&compilation.to_ir().unwrap()).unwrap()).unwrap(),
        "1\n2\n1\n4\n"
    );
}

#[test]
fn rejects_visibility_initialization_cycles_and_namespace_errors_at_their_source() {
    let w = Workspace::new();
    let valid = "pub type Value { n: u64, } type Private { n: u64, } pub fn value() -> u64 { return helper(); } fn helper() -> u64 { return 7; }";
    for (library, entry, reason, file) in [
        (
            valid,
            "import \"lib.prim\" as lib; print(lib::helper());",
            "private function",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; x: lib::Private = lib::Private { n: 1 };",
            "private type",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; print(helper());",
            "unknown function",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; print(other::value());",
            "unknown import alias",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; print(lib::missing());",
            "unknown function",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; print(lib::value);",
            "not variable values",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; import \"lib.prim\" as lib;",
            "duplicate import alias",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; fn lib() -> void {}",
            "conflicts with an import alias",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; fn f(lib: i64) -> void {}",
            "conflicts with an import alias",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as lib; mut lib: i64 = 0;",
            "conflicts with an import alias",
            "main.prim",
        ),
        (
            valid,
            "import \"lib.prim\" as i64;",
            "reserved",
            "main.prim",
        ),
        (
            "print(99);",
            "import \"lib.prim\" as lib;",
            "only imports, types, and functions",
            "lib.prim",
        ),
        (
            "import \"main.prim\" as root;",
            "import \"lib.prim\" as lib;",
            "cyclic import",
            "lib.prim",
        ),
        (
            "type Hidden { x: i64, } pub fn leak() -> Hidden { return Hidden { x: 1 }; }",
            "import \"lib.prim\" as lib;",
            "public signature",
            "lib.prim",
        ),
        (
            "type Hidden { x: i64, } pub type Leak { x: [Hidden; 1], }",
            "import \"lib.prim\" as lib;",
            "public signature",
            "lib.prim",
        ),
        (
            "pub type Same { x: i64, } fn Same() -> i64 { return 1; }",
            "import \"lib.prim\" as lib; print(lib::Same());",
            "private function",
            "main.prim",
        ),
        (
            "pub fn i64() -> i64 { return 1; }",
            "import \"lib.prim\" as lib;",
            "reserved",
            "lib.prim",
        ),
        (
            "pub fn f() -> void { f(); }",
            "import \"lib.prim\" as lib;",
            "recursive",
            "lib.prim",
        ),
        (
            "pub fn f() -> void { print(missing); }",
            "import \"lib.prim\" as lib;",
            "unknown binding",
            "lib.prim",
        ),
        (
            "pub fn f() -> void { print(\"\\q\"); }",
            "import \"lib.prim\" as lib;",
            "escape",
            "lib.prim",
        ),
    ] {
        w.put("lib.prim", library);
        let path = w.put("main.prim", entry);
        let (message, span, name) = match modules::load(&path) {
            Err(error) => {
                let span = error.diagnostic.primary_span().unwrap();
                (
                    error.diagnostic.message().to_owned(),
                    span,
                    error
                        .sources
                        .get(span.source_id())
                        .map(|s| s.name().to_owned()),
                )
            }
            Ok(compilation) => {
                let error = compilation.to_ir().unwrap_err();
                let span = error.primary_span().unwrap();
                (
                    error.message().to_owned(),
                    span,
                    compilation
                        .sources
                        .get(span.source_id())
                        .map(|s| s.name().to_owned()),
                )
            }
        };
        assert!(message.contains(reason), "{entry}: {message}");
        assert!(name.unwrap().ends_with(file), "{entry}: {span:?}");
    }
}

#[test]
fn paths_and_declarations_are_explicit_and_fail_before_overwriting_output() {
    let w = Workspace::new();
    for source in [
        "import \"absent.prim\" as missing;",
        "import \"/absolute.prim\" as missing;",
        "import \"C:/absolute.prim\" as missing;",
        "import \"dir\\\\file.prim\" as missing;",
        "import \"bad\\0.prim\" as missing;",
        "import \"file.txt\" as missing;",
        "import \"absent.prim\";",
        "print(1); import \"lib.prim\" as lib;",
        "pub x: i64 = 1;",
        "pub fn f() -> void { import \"lib.prim\" as lib; }",
    ] {
        let entry = w.put("main.prim", source);
        w.put("output.c", "keep");
        let output = w.cli("emit-c", &entry, &["-o", "output.c"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read_to_string(w.0.join("output.c")).unwrap(), "keep");
    }
    assert!(
        primer_lang::compile("import \"lib.prim\" as lib;")
            .unwrap_err()
            .message()
            .contains("modules::load")
    );
    assert!(
        primer_lang::compile("pub fn f() -> void {}")
            .unwrap_err()
            .message()
            .contains("modules::load")
    );
    let entry = w.put(
        "main.prim",
        "type Hidden { n: i64, } pub type Leak { v: Hidden, }",
    );
    assert!(
        modules::load(&entry)
            .unwrap_err()
            .render()
            .contains("public signature")
    );
}

#[test]
fn module_entry_syntax_errors_keep_file_identity_and_deep_imports_are_bounded() {
    let w = Workspace::new();
    let entry = w.put("main.prim", "import \"lib.prim\" as lib; print(1 + );");
    let error = modules::load(&entry).unwrap_err();
    assert_eq!(
        error.diagnostic.primary_span().unwrap().source_id().index(),
        1
    );
    assert!(error.render().contains("main.prim:1:"));
    for index in 0..129 {
        w.put(
            &format!("m{index}.prim"),
            &format!("import \"m{}.prim\" as next;", index + 1),
        );
    }
    w.put("m129.prim", "");
    let error = modules::load(&w.0.join("m0.prim")).unwrap_err();
    assert!(error.render().contains("import nesting exceeds 128"));
    w.put("m128.prim", "");
    assert_eq!(
        modules::load(&w.0.join("m0.prim"))
            .unwrap()
            .sources
            .files()
            .count(),
        129
    );
}

#[test]
#[cfg(unix)]
fn symlink_imports_share_identity_and_resolve_from_the_target_directory() {
    let w = Workspace::new();
    w.put(
        "actual/value.prim",
        "pub type Value { n: i64, } pub fn make() -> Value { return Value { n: 7 }; }",
    );
    w.put(
        "actual/logic.prim",
        "import \"value.prim\" as v; pub fn make() -> v::Value { return v::make(); }",
    );
    std::os::unix::fs::symlink(w.0.join("actual/logic.prim"), w.0.join("link.prim")).unwrap();
    let entry = w.put("main.prim", "import \"link.prim\" as a; import \"actual/logic.prim\" as b; print(a::make().n); print(b::make().n);");
    let compilation = modules::load(&entry).unwrap();
    assert_eq!(compilation.sources.files().count(), 3);
    for source in compilation.sources.files() {
        assert_eq!(fs::read_to_string(source.name()).unwrap(), source.text());
    }
    assert_eq!(
        run_bytecode(&bytecode::lower(&compilation.to_ir().unwrap()).unwrap()).unwrap(),
        "7\n7\n"
    );
}

#[test]
fn manifest_preserves_dependency_text_and_cli_failures_identify_the_definition_file() {
    let w = Workspace::new();
    let library = "// 日本語\r\npub fn divide(x: i64) -> i64 { print(\"先行\\0\\r\\n\"); return 10 / x; }\r\n";
    w.put("lib.prim", library);
    let entry = w.put(
        "main.prim",
        "import \"lib.prim\" as lib; print(lib::divide(0));",
    );
    let compilation = modules::load(&entry).unwrap();
    let error = run_bytecode(&bytecode::lower(&compilation.to_ir().unwrap()).unwrap()).unwrap_err();
    let failure = error.runtime_failure().unwrap();
    assert_eq!(compilation.sources.slice(failure.span), Some("10 / x"));
    assert_eq!(failure.span.source_id().index(), 2);
    let output = w.cli("run", &entry, &["--diagnostic-format", "runtime-v1"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, "先行\0\r\n\n".as_bytes());
    assert_eq!(
        String::from_utf8(output.stderr)
            .unwrap()
            .replace("\r\n", "\n"),
        format!("primer: {}\n", failure.record())
    );
    let human = w.cli("run", &entry, &[]);
    assert!(
        String::from_utf8(human.stderr)
            .unwrap()
            .contains("lib.prim:2:")
    );
    let output = w.cli("emit-sources", &entry, &[]);
    assert!(output.status.success());
    assert_eq!(output.stdout, compilation.source_manifest().as_bytes());
    let node = std::env::var_os("PRIMER_TEST_NODE").unwrap_or_else(|| "node".into());
    fs::write(w.0.join("sources.json"), output.stdout).unwrap();
    let output = process::bounded_output(Command::new(node).arg("-e").arg("const fs=require('fs');const m=JSON.parse(fs.readFileSync(process.argv[1],'utf8'));if(m.schema!=='primer-sources-v1'||m.files.length!==2||m.files[1].id!==2)process.exit(1);process.stdout.write(m.files[1].text);").arg(w.0.join("sources.json")), &w.0, "manifest", Duration::from_secs(10)).unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, library.as_bytes());
}
