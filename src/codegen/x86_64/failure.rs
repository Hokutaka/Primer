use super::{Target, ir::Origin};
use crate::runtime::{FailureCode, RuntimeFailure};

/// 停止分岐だけに記録処理を置きます。正常経路に可変な「現在位置」を作りません。
pub(super) struct Reporter {
    pub origin: Origin,
    target: Target,
    count: usize,
    pub data: String,
}

impl Reporter {
    pub fn new(target: Target) -> Self {
        Self {
            origin: Origin::Synthetic,
            target,
            count: 0,
            data: String::new(),
        }
    }

    pub fn emit(&mut self, code: FailureCode, output: &mut String) {
        let Origin::Source { node_id, span } = self.origin else {
            panic!("language failure requires a source origin");
        };
        let record = format!(
            "primer: {}\n",
            RuntimeFailure {
                code,
                node_id,
                span
            }
            .record()
        );
        let label = format!(".Lprimer_failure_{}", self.count);
        self.count += 1;
        self.data.push_str(&format!(
            "{label}:\n  .asciz \"{}\"\n",
            record.replace('\n', "\\n")
        ));
        // 既に実行したprintを保持してから、明示したOSのCRTでstderrへ記録します。
        // stderrへの書き込みに失敗しても、元の異常停止を必ず実行します。
        if self.target.is_linux() {
            output.push_str(&format!("  xorl %edi, %edi\n  callq fflush\n  movl $2, %edi\n  leaq {label}(%rip), %rsi\n  movl ${}, %edx\n  callq write\n", record.len()));
        } else {
            output.push_str(&format!("  xorl %ecx, %ecx\n  callq fflush\n  movl $2, %ecx\n  leaq {label}(%rip), %rdx\n  movl ${}, %r8d\n  callq _write\n", record.len()));
        }
        output.push_str("  ud2\n");
    }
}
