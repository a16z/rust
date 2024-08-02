use clippy_config::msrvs::{self, Msrv};
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::in_constant;
use clippy_utils::source::snippet_opt;
use clippy_utils::sugg::Sugg;
use clippy_utils::ty::is_isize_or_usize;
use rustc_errors::Applicability;
use rustc_hir::{Expr, QPath, TyKind};
use rustc_lint::LateContext;
use rustc_middle::ty::{self, FloatTy, Ty};
use rustc_span::hygiene;

use super::{utils, CAST_LOSSLESS};

pub(super) fn check(
    cx: &LateContext<'_>,
    expr: &Expr<'_>,
    cast_from_expr: &Expr<'_>,
    cast_from: Ty<'_>,
    cast_to: Ty<'_>,
    cast_to_hir: &rustc_hir::Ty<'_>,
    msrv: &Msrv,
) {
    if !should_lint(cx, expr, cast_from, cast_to, msrv) {
        return;
    }

    span_lint_and_then(
        cx,
        CAST_LOSSLESS,
        expr.span,
        format!("casts from `{cast_from}` to `{cast_to}` can be expressed infallibly using `From`"),
        |diag| {
            diag.help("an `as` cast can become silently lossy if the types change in the future");
            let mut applicability = Applicability::MachineApplicable;
            let from_sugg = Sugg::hir_with_context(cx, cast_from_expr, expr.span.ctxt(), "<from>", &mut applicability);
            let Some(ty) = snippet_opt(cx, hygiene::walk_chain(cast_to_hir.span, expr.span.ctxt())) else {
                return;
            };
            match cast_to_hir.kind {
                TyKind::Infer => {
                    diag.span_suggestion_verbose(
                        expr.span,
                        "use `Into::into` instead",
                        format!("{}.into()", from_sugg.maybe_par()),
                        applicability,
                    );
                },
                // Don't suggest `A<_>::B::From(x)` or `macro!()::from(x)`
                kind if matches!(kind, TyKind::Path(QPath::Resolved(_, path)) if path.segments.iter().any(|s| s.args.is_some()))
                    || !cast_to_hir.span.eq_ctxt(expr.span) =>
                {
                    diag.span_suggestion_verbose(
                        expr.span,
                        format!("use `<{ty}>::from` instead"),
                        format!("<{ty}>::from({from_sugg})"),
                        applicability,
                    );
                },
                _ => {
                    diag.span_suggestion_verbose(
                        expr.span,
                        format!("use `{ty}::from` instead"),
                        format!("{ty}::from({from_sugg})"),
                        applicability,
                    );
                },
            }
        },
    );
}

fn should_lint(cx: &LateContext<'_>, expr: &Expr<'_>, cast_from: Ty<'_>, cast_to: Ty<'_>, msrv: &Msrv) -> bool {
    // Do not suggest using From in consts/statics until it is valid to do so (see #2267).
    if in_constant(cx, expr.hir_id) {
        return false;
    }

    match (cast_from.is_integral(), cast_to.is_integral()) {
        (true, true) => {
            let cast_signed_to_unsigned = cast_from.is_signed() && !cast_to.is_signed();
            let from_nbits = utils::int_ty_to_nbits(cast_from, cx.tcx);
            let to_nbits = utils::int_ty_to_nbits(cast_to, cx.tcx);
            !is_isize_or_usize(cast_from)
                && !is_isize_or_usize(cast_to)
                && from_nbits < to_nbits
                && !cast_signed_to_unsigned
        },

        (true, false) => {
            let from_nbits = utils::int_ty_to_nbits(cast_from, cx.tcx);
            let to_nbits = if let ty::Float(FloatTy::F32) = cast_to.kind() {
                32
            } else {
                64
            };
            !is_isize_or_usize(cast_from) && from_nbits < to_nbits
        },
        (false, true) if matches!(cast_from.kind(), ty::Bool) && msrv.meets(msrvs::FROM_BOOL) => true,
        (_, _) => {
            matches!(cast_from.kind(), ty::Float(FloatTy::F32)) && matches!(cast_to.kind(), ty::Float(FloatTy::F64))
        },
    }
}
