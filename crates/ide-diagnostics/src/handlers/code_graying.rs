use std::rc::Rc;

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};
use hir::diagnostics::CodeGraying;

pub(crate) fn code_graying(ctx: &DiagnosticsContext<'_>, d: &Box<CodeGraying>) -> Diagnostic {
    let range = match &d.span {
        either::Either::Left(l) => match &l.value {
            either::Either::Left(l1) => l1.text_range(),
            either::Either::Right(l2) => l2.text_range(),
        },
        either::Either::Right(r) => r.value.text_range(),
    };

    Diagnostic {
        code: DiagnosticCode::Ra("Code grayin", crate::Severity::Error),
        message: "GRAYING".into(),
        range,
        severity: crate::Severity::Error,
        unused: true,
        experimental: false,
        fixes: None,
        main_node: None,
    }
}
