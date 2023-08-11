use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};
use hir::diagnostics::CodeGraying;

pub(crate) fn code_graying(ctx: &DiagnosticsContext<'_>, d: &Box<CodeGraying>) -> Diagnostic {
    let range = match &d.span {
        either::Either::Left(l) => match &l.value {
            either::Either::Left(l1) => {
                eprintln!("{:?}", l1);
                l1.text_range()
            }
            either::Either::Right(l2) => {
                eprintln!("{:?}", l2);
                l2.text_range()
            }
        },
        either::Either::Right(r) => {
            eprintln!("{:?}", r);
            r.value.text_range()
        }
    };

    Diagnostic {
        code: DiagnosticCode::Ra("Code grayin", crate::Severity::Error),
        message: "GRAYING".into(),
        range,
        severity: crate::Severity::Warning,
        unused: true,
        experimental: false,
        fixes: None,
        main_node: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn no_such_field_diagnostics() {
        check_diagnostics(
            r#"
struct S { foo: i32, bar: () }
impl S {
    fn new() -> S {
        S {
      //^ 💡 error: missing structure fields:
      //|    - bar
            foo: 92,
            baz: 62,
          //^^^^^^^ 💡 error: no such field
        }
    }
}
"#,
        );
    }
}
