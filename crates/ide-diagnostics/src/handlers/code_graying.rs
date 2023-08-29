use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};
use hir::diagnostics::{CodeGraying, CodeUngraying};

pub(crate) fn code_graying(ctx: &DiagnosticsContext<'_>, d: &Box<CodeGraying>) -> Diagnostic {
    let range = match &d.span {
        either::Either::Left(span) => span.value.syntax_node_ptr().text_range(),
        either::Either::Right(span) => match &span.value {
            either::Either::Left(sl) => sl.text_range(),
            either::Either::Right(sr) => sr.text_range(),
        },
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

pub(crate) fn code_ungraying(ctx: &DiagnosticsContext<'_>, d: &Box<CodeUngraying>) -> Diagnostic {
    let range = match &d.span {
        either::Either::Left(span) => span.value.syntax_node_ptr().text_range(),
        either::Either::Right(span) => match &span.value {
            either::Either::Left(sl) => sl.text_range(),
            either::Either::Right(sr) => sr.text_range(),
        },
    };

    Diagnostic {
        code: DiagnosticCode::Ra("Code ungrayin", crate::Severity::Error),
        message: "UNGRAYING".into(),
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
    fn deneme() {
        check_diagnostics(
            r#"
fn abc() -> i32 {
    let i = 3 ;
    

    if i > 5 {
        return 4;
        let i = 5;
    } else {
        panic!("ABC");
    }

    3
}
"#,
        );
    }
}
