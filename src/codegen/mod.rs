pub mod c;
pub mod llvm;
pub mod qbe;
pub mod wat;
pub mod x86_64_win;

pub use c::emit_c;
pub use llvm::emit_llvm;
pub use qbe::emit_qbe;
pub use wat::emit_wat;
pub use x86_64_win::emit_x86_64_win_asm;
