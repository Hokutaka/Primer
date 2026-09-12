// 生成物を比較するテストではホストOSからABIを選ばず、Linuxを明示します。
// 実行テストは実行環境に対応したターゲットを個別に指定します。
pub fn compile(source: &str) -> Result<String, primer_lang::diagnostic::Diagnostic> {
    primer_lang::compile_to_llvm_with_target(
        source,
        Some(primer_lang::codegen::llvm::Target::X86_64UnknownLinuxGnu),
    )
}
