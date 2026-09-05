use std::fmt::Write;

use super::ir::{Module, Type};

pub(super) fn uses_type(module: &Module) -> bool {
    fn contains(ty: &Type) -> bool {
        match ty {
            Type::String => true,
            Type::Array { element, .. } => contains(element),
            Type::Bool | Type::I64 | Type::Float | Type::Double | Type::Named(_) => false,
        }
    }
    module
        .type_definitions
        .iter()
        .any(|definition| definition.fields.iter().any(|field| contains(&field.ty)))
        || module.array_types.iter().any(contains)
        || module.functions.iter().any(|function| {
            function.return_type.as_ref().is_some_and(contains)
                || function
                    .parameters
                    .iter()
                    .any(|parameter| contains(&parameter.ty))
        })
}

/// C文字列リテラルの静的な保存期間を使い、関数から返してもデータを失いません。
pub(super) fn literal(value: &str, output: &mut String) {
    output.push_str("(primer_string){ (const unsigned char *)\"");
    // 全バイトを固定3桁の8進表記にし、Cの文字コード設定や後続文字に左右されません。
    for byte in value.bytes() {
        write!(output, "\\{byte:03o}").unwrap();
    }
    write!(output, "\", {} }}", value.len()).unwrap();
}

pub(super) const SUPPORT: &str = r#"typedef struct primer_string {
    const unsigned char *data;
    size_t length;
} primer_string;

static inline bool primer_string_equal(primer_string left, primer_string right) {
    return left.length == right.length &&
        (left.length == 0 || memcmp(left.data, right.data, left.length) == 0);
}

static inline void primer_print_string(primer_string value) {
    fwrite(value.data, 1, value.length, stdout);
    fputc('\n', stdout);
}

"#;
