//! This module handles checking if the span given is from a proc-macro or not.
//!
//! Proc-macros are capable of setting the span of every token they output to a few possible spans.
//! This includes spans we can detect easily as coming from a proc-macro (e.g. the call site
//! or the def site), and spans we can't easily detect as such (e.g. the span of any token
//! passed into the proc macro). This capability means proc-macros are capable of generating code
//! with a span that looks like it was written by the user, but which should not be linted by clippy
//! as it was generated by an external macro.
//!
//! That brings us to this module. The current approach is to determine a small bit of text which
//! must exist at both the start and the end of an item (e.g. an expression or a path) assuming the
//! code was written, and check if the span contains that text. Note this will only work correctly
//! if the span is not from a `macro_rules` based macro.

use rustc_ast::ast::{AttrKind, Attribute, IntTy, LitIntType, LitKind, StrStyle, TraitObjectSyntax, UintTy};
use rustc_ast::token::CommentKind;
use rustc_ast::AttrStyle;
use rustc_hir::intravisit::FnKind;
use rustc_hir::{
    Block, BlockCheckMode, Body, Closure, Destination, Expr, ExprKind, FieldDef, FnHeader, FnRetTy, HirId, Impl,
    ImplItem, ImplItemKind, IsAuto, Item, ItemKind, Lit, LoopSource, MatchSource, MutTy, Node, Path, QPath, Safety,
    TraitItem, TraitItemKind, Ty, TyKind, UnOp, UnsafeSource, Variant, VariantData, YieldSource,
};
use rustc_lint::{LateContext, LintContext};
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;
use rustc_span::symbol::{kw, Ident};
use rustc_span::{Span, Symbol};
use rustc_target::spec::abi::Abi;

/// The search pattern to look for. Used by `span_matches_pat`
#[derive(Clone)]
pub enum Pat {
    /// A single string.
    Str(&'static str),
    /// Any of the given strings.
    MultiStr(&'static [&'static str]),
    /// Any of the given strings.
    OwnedMultiStr(Vec<String>),
    /// The string representation of the symbol.
    Sym(Symbol),
    /// Any decimal or hexadecimal digit depending on the location.
    Num,
}

/// Checks if the start and the end of the span's text matches the patterns. This will return false
/// if the span crosses multiple files or if source is not available.
fn span_matches_pat(sess: &Session, span: Span, start_pat: Pat, end_pat: Pat) -> bool {
    let pos = sess.source_map().lookup_byte_offset(span.lo());
    let Some(ref src) = pos.sf.src else {
        return false;
    };
    let end = span.hi() - pos.sf.start_pos;
    src.get(pos.pos.0 as usize..end.0 as usize).map_or(false, |s| {
        // Spans can be wrapped in a mixture or parenthesis, whitespace, and trailing commas.
        let start_str = s.trim_start_matches(|c: char| c.is_whitespace() || c == '(');
        let end_str = s.trim_end_matches(|c: char| c.is_whitespace() || c == ')' || c == ',');
        (match start_pat {
            Pat::Str(text) => start_str.starts_with(text),
            Pat::MultiStr(texts) => texts.iter().any(|s| start_str.starts_with(s)),
            Pat::OwnedMultiStr(texts) => texts.iter().any(|s| start_str.starts_with(s)),
            Pat::Sym(sym) => start_str.starts_with(sym.as_str()),
            Pat::Num => start_str.as_bytes().first().map_or(false, u8::is_ascii_digit),
        } && match end_pat {
            Pat::Str(text) => end_str.ends_with(text),
            Pat::MultiStr(texts) => texts.iter().any(|s| start_str.ends_with(s)),
            Pat::OwnedMultiStr(texts) => texts.iter().any(|s| start_str.starts_with(s)),
            Pat::Sym(sym) => end_str.ends_with(sym.as_str()),
            Pat::Num => end_str.as_bytes().last().map_or(false, u8::is_ascii_hexdigit),
        })
    })
}

/// Get the search patterns to use for the given literal
fn lit_search_pat(lit: &LitKind) -> (Pat, Pat) {
    match lit {
        LitKind::Str(_, StrStyle::Cooked) => (Pat::Str("\""), Pat::Str("\"")),
        LitKind::Str(_, StrStyle::Raw(0)) => (Pat::Str("r"), Pat::Str("\"")),
        LitKind::Str(_, StrStyle::Raw(_)) => (Pat::Str("r#"), Pat::Str("#")),
        LitKind::ByteStr(_, StrStyle::Cooked) => (Pat::Str("b\""), Pat::Str("\"")),
        LitKind::ByteStr(_, StrStyle::Raw(0)) => (Pat::Str("br\""), Pat::Str("\"")),
        LitKind::ByteStr(_, StrStyle::Raw(_)) => (Pat::Str("br#\""), Pat::Str("#")),
        LitKind::Byte(_) => (Pat::Str("b'"), Pat::Str("'")),
        LitKind::Char(_) => (Pat::Str("'"), Pat::Str("'")),
        LitKind::Int(_, LitIntType::Signed(IntTy::Isize)) => (Pat::Num, Pat::Str("isize")),
        LitKind::Int(_, LitIntType::Unsigned(UintTy::Usize)) => (Pat::Num, Pat::Str("usize")),
        LitKind::Int(..) => (Pat::Num, Pat::Num),
        LitKind::Float(..) => (Pat::Num, Pat::Str("")),
        LitKind::Bool(true) => (Pat::Str("true"), Pat::Str("true")),
        LitKind::Bool(false) => (Pat::Str("false"), Pat::Str("false")),
        _ => (Pat::Str(""), Pat::Str("")),
    }
}

/// Get the search patterns to use for the given path
fn qpath_search_pat(path: &QPath<'_>) -> (Pat, Pat) {
    match path {
        QPath::Resolved(ty, path) => {
            let start = if ty.is_some() {
                Pat::Str("<")
            } else {
                path.segments.first().map_or(Pat::Str(""), |seg| {
                    if seg.ident.name == kw::PathRoot {
                        Pat::Str("::")
                    } else {
                        Pat::Sym(seg.ident.name)
                    }
                })
            };
            let end = path.segments.last().map_or(Pat::Str(""), |seg| {
                if seg.args.is_some() {
                    Pat::Str(">")
                } else {
                    Pat::Sym(seg.ident.name)
                }
            });
            (start, end)
        },
        QPath::TypeRelative(_, name) => (Pat::Str(""), Pat::Sym(name.ident.name)),
        QPath::LangItem(..) => (Pat::Str(""), Pat::Str("")),
    }
}

fn path_search_pat(path: &Path<'_>) -> (Pat, Pat) {
    let (head, tail) = match path.segments {
        [head, .., tail] => (head, tail),
        [p] => (p, p),
        [] => return (Pat::Str(""), Pat::Str("")),
    };
    (
        if head.ident.name == kw::PathRoot {
            Pat::Str("::")
        } else {
            Pat::Sym(head.ident.name)
        },
        if tail.args.is_some() {
            Pat::Str(">")
        } else {
            Pat::Sym(tail.ident.name)
        },
    )
}

/// Get the search patterns to use for the given expression
fn expr_search_pat(tcx: TyCtxt<'_>, e: &Expr<'_>) -> (Pat, Pat) {
    match e.kind {
        ExprKind::ConstBlock(_) => (Pat::Str("const"), Pat::Str("}")),
        // Parenthesis are trimmed from the text before the search patterns are matched.
        // See: `span_matches_pat`
        ExprKind::Tup([]) => (Pat::Str(")"), Pat::Str("(")),
        ExprKind::Unary(UnOp::Deref, e) => (Pat::Str("*"), expr_search_pat(tcx, e).1),
        ExprKind::Unary(UnOp::Not, e) => (Pat::Str("!"), expr_search_pat(tcx, e).1),
        ExprKind::Unary(UnOp::Neg, e) => (Pat::Str("-"), expr_search_pat(tcx, e).1),
        ExprKind::Lit(lit) => lit_search_pat(&lit.node),
        ExprKind::Array(_) | ExprKind::Repeat(..) => (Pat::Str("["), Pat::Str("]")),
        ExprKind::Call(e, []) | ExprKind::MethodCall(_, e, [], _) => (expr_search_pat(tcx, e).0, Pat::Str("(")),
        ExprKind::Call(first, [.., last])
        | ExprKind::MethodCall(_, first, [.., last], _)
        | ExprKind::Binary(_, first, last)
        | ExprKind::Tup([first, .., last])
        | ExprKind::Assign(first, last, _)
        | ExprKind::AssignOp(_, first, last) => (expr_search_pat(tcx, first).0, expr_search_pat(tcx, last).1),
        ExprKind::Tup([e]) | ExprKind::DropTemps(e) => expr_search_pat(tcx, e),
        ExprKind::Cast(e, _) | ExprKind::Type(e, _) => (expr_search_pat(tcx, e).0, Pat::Str("")),
        ExprKind::Let(let_expr) => (Pat::Str("let"), expr_search_pat(tcx, let_expr.init).1),
        ExprKind::If(..) => (Pat::Str("if"), Pat::Str("}")),
        ExprKind::Loop(_, Some(_), _, _) | ExprKind::Block(_, Some(_)) => (Pat::Str("'"), Pat::Str("}")),
        ExprKind::Loop(_, None, LoopSource::Loop, _) => (Pat::Str("loop"), Pat::Str("}")),
        ExprKind::Loop(_, None, LoopSource::While, _) => (Pat::Str("while"), Pat::Str("}")),
        ExprKind::Loop(_, None, LoopSource::ForLoop, _) | ExprKind::Match(_, _, MatchSource::ForLoopDesugar) => {
            (Pat::Str("for"), Pat::Str("}"))
        },
        ExprKind::Match(_, _, MatchSource::Normal) => (Pat::Str("match"), Pat::Str("}")),
        ExprKind::Match(e, _, MatchSource::TryDesugar(_)) => (expr_search_pat(tcx, e).0, Pat::Str("?")),
        ExprKind::Match(e, _, MatchSource::AwaitDesugar) | ExprKind::Yield(e, YieldSource::Await { .. }) => {
            (expr_search_pat(tcx, e).0, Pat::Str("await"))
        },
        ExprKind::Closure(&Closure { body, .. }) => (Pat::Str(""), expr_search_pat(tcx, tcx.hir().body(body).value).1),
        ExprKind::Block(
            Block {
                rules: BlockCheckMode::UnsafeBlock(UnsafeSource::UserProvided),
                ..
            },
            None,
        ) => (Pat::Str("unsafe"), Pat::Str("}")),
        ExprKind::Block(_, None) => (Pat::Str("{"), Pat::Str("}")),
        ExprKind::Field(e, name) => (expr_search_pat(tcx, e).0, Pat::Sym(name.name)),
        ExprKind::Index(e, _, _) => (expr_search_pat(tcx, e).0, Pat::Str("]")),
        ExprKind::Path(ref path) => qpath_search_pat(path),
        ExprKind::AddrOf(_, _, e) => (Pat::Str("&"), expr_search_pat(tcx, e).1),
        ExprKind::Break(Destination { label: None, .. }, None) => (Pat::Str("break"), Pat::Str("break")),
        ExprKind::Break(Destination { label: Some(name), .. }, None) => (Pat::Str("break"), Pat::Sym(name.ident.name)),
        ExprKind::Break(_, Some(e)) => (Pat::Str("break"), expr_search_pat(tcx, e).1),
        ExprKind::Continue(Destination { label: None, .. }) => (Pat::Str("continue"), Pat::Str("continue")),
        ExprKind::Continue(Destination { label: Some(name), .. }) => (Pat::Str("continue"), Pat::Sym(name.ident.name)),
        ExprKind::Ret(None) => (Pat::Str("return"), Pat::Str("return")),
        ExprKind::Ret(Some(e)) => (Pat::Str("return"), expr_search_pat(tcx, e).1),
        ExprKind::Struct(path, _, _) => (qpath_search_pat(path).0, Pat::Str("}")),
        ExprKind::Yield(e, YieldSource::Yield) => (Pat::Str("yield"), expr_search_pat(tcx, e).1),
        _ => (Pat::Str(""), Pat::Str("")),
    }
}

fn fn_header_search_pat(header: FnHeader) -> Pat {
    if header.is_async() {
        Pat::Str("async")
    } else if header.is_const() {
        Pat::Str("const")
    } else if header.is_unsafe() {
        Pat::Str("unsafe")
    } else if header.abi != Abi::Rust {
        Pat::Str("extern")
    } else {
        Pat::MultiStr(&["fn", "extern"])
    }
}

fn item_search_pat(item: &Item<'_>) -> (Pat, Pat) {
    let (start_pat, end_pat) = match &item.kind {
        ItemKind::ExternCrate(_) => (Pat::Str("extern"), Pat::Str(";")),
        ItemKind::Static(..) => (Pat::Str("static"), Pat::Str(";")),
        ItemKind::Const(..) => (Pat::Str("const"), Pat::Str(";")),
        ItemKind::Fn(sig, ..) => (fn_header_search_pat(sig.header), Pat::Str("")),
        ItemKind::ForeignMod { .. } => (Pat::Str("extern"), Pat::Str("}")),
        ItemKind::TyAlias(..) | ItemKind::OpaqueTy(_) => (Pat::Str("type"), Pat::Str(";")),
        ItemKind::Enum(..) => (Pat::Str("enum"), Pat::Str("}")),
        ItemKind::Struct(VariantData::Struct { .. }, _) => (Pat::Str("struct"), Pat::Str("}")),
        ItemKind::Struct(..) => (Pat::Str("struct"), Pat::Str(";")),
        ItemKind::Union(..) => (Pat::Str("union"), Pat::Str("}")),
        ItemKind::Trait(_, Safety::Unsafe, ..)
        | ItemKind::Impl(Impl {
            safety: Safety::Unsafe, ..
        }) => (Pat::Str("unsafe"), Pat::Str("}")),
        ItemKind::Trait(IsAuto::Yes, ..) => (Pat::Str("auto"), Pat::Str("}")),
        ItemKind::Trait(..) => (Pat::Str("trait"), Pat::Str("}")),
        ItemKind::Impl(_) => (Pat::Str("impl"), Pat::Str("}")),
        _ => return (Pat::Str(""), Pat::Str("")),
    };
    if item.vis_span.is_empty() {
        (start_pat, end_pat)
    } else {
        (Pat::Str("pub"), end_pat)
    }
}

fn trait_item_search_pat(item: &TraitItem<'_>) -> (Pat, Pat) {
    match &item.kind {
        TraitItemKind::Const(..) => (Pat::Str("const"), Pat::Str(";")),
        TraitItemKind::Type(..) => (Pat::Str("type"), Pat::Str(";")),
        TraitItemKind::Fn(sig, ..) => (fn_header_search_pat(sig.header), Pat::Str("")),
    }
}

fn impl_item_search_pat(item: &ImplItem<'_>) -> (Pat, Pat) {
    let (start_pat, end_pat) = match &item.kind {
        ImplItemKind::Const(..) => (Pat::Str("const"), Pat::Str(";")),
        ImplItemKind::Type(..) => (Pat::Str("type"), Pat::Str(";")),
        ImplItemKind::Fn(sig, ..) => (fn_header_search_pat(sig.header), Pat::Str("")),
    };
    if item.vis_span.is_empty() {
        (start_pat, end_pat)
    } else {
        (Pat::Str("pub"), end_pat)
    }
}

fn field_def_search_pat(def: &FieldDef<'_>) -> (Pat, Pat) {
    if def.vis_span.is_empty() {
        if def.is_positional() {
            (Pat::Str(""), Pat::Str(""))
        } else {
            (Pat::Sym(def.ident.name), Pat::Str(""))
        }
    } else {
        (Pat::Str("pub"), Pat::Str(""))
    }
}

fn variant_search_pat(v: &Variant<'_>) -> (Pat, Pat) {
    match v.data {
        VariantData::Struct { .. } => (Pat::Sym(v.ident.name), Pat::Str("}")),
        VariantData::Tuple(..) => (Pat::Sym(v.ident.name), Pat::Str("")),
        VariantData::Unit(..) => (Pat::Sym(v.ident.name), Pat::Sym(v.ident.name)),
    }
}

fn fn_kind_pat(tcx: TyCtxt<'_>, kind: &FnKind<'_>, body: &Body<'_>, hir_id: HirId) -> (Pat, Pat) {
    let (start_pat, end_pat) = match kind {
        FnKind::ItemFn(.., header) => (fn_header_search_pat(*header), Pat::Str("")),
        FnKind::Method(.., sig) => (fn_header_search_pat(sig.header), Pat::Str("")),
        FnKind::Closure => return (Pat::Str(""), expr_search_pat(tcx, body.value).1),
    };
    let start_pat = match tcx.hir_node(hir_id) {
        Node::Item(Item { vis_span, .. }) | Node::ImplItem(ImplItem { vis_span, .. }) => {
            if vis_span.is_empty() {
                start_pat
            } else {
                Pat::Str("pub")
            }
        },
        Node::TraitItem(_) => start_pat,
        _ => Pat::Str(""),
    };
    (start_pat, end_pat)
}

fn attr_search_pat(attr: &Attribute) -> (Pat, Pat) {
    match attr.kind {
        AttrKind::Normal(..) => {
            if let Some(ident) = attr.ident() {
                // TODO: I feel like it's likely we can use `Cow` instead but this will require quite a bit of
                // refactoring
                // NOTE: This will likely have false positives, like `allow = 1`
                (
                    Pat::OwnedMultiStr(vec![ident.to_string(), "#".to_owned()]),
                    Pat::Str(""),
                )
            } else {
                (Pat::Str("#"), Pat::Str("]"))
            }
        },
        AttrKind::DocComment(_kind @ CommentKind::Line, ..) => {
            if matches!(attr.style, AttrStyle::Outer) {
                (Pat::Str("///"), Pat::Str(""))
            } else {
                (Pat::Str("//!"), Pat::Str(""))
            }
        },
        AttrKind::DocComment(_kind @ CommentKind::Block, ..) => {
            if matches!(attr.style, AttrStyle::Outer) {
                (Pat::Str("/**"), Pat::Str("*/"))
            } else {
                (Pat::Str("/*!"), Pat::Str("*/"))
            }
        },
    }
}

fn ty_search_pat(ty: &Ty<'_>) -> (Pat, Pat) {
    match ty.kind {
        TyKind::Slice(..) | TyKind::Array(..) => (Pat::Str("["), Pat::Str("]")),
        TyKind::Ptr(MutTy { ty, .. }) => (Pat::Str("*"), ty_search_pat(ty).1),
        TyKind::Ref(_, MutTy { ty, .. }) => (Pat::Str("&"), ty_search_pat(ty).1),
        TyKind::BareFn(bare_fn) => (
            if bare_fn.safety == Safety::Unsafe {
                Pat::Str("unsafe")
            } else if bare_fn.abi != Abi::Rust {
                Pat::Str("extern")
            } else {
                Pat::MultiStr(&["fn", "extern"])
            },
            match bare_fn.decl.output {
                FnRetTy::DefaultReturn(_) => {
                    if let [.., ty] = bare_fn.decl.inputs {
                        ty_search_pat(ty).1
                    } else {
                        Pat::Str("(")
                    }
                },
                FnRetTy::Return(ty) => ty_search_pat(ty).1,
            },
        ),
        TyKind::Never => (Pat::Str("!"), Pat::Str("!")),
        // Parenthesis are trimmed from the text before the search patterns are matched.
        // See: `span_matches_pat`
        TyKind::Tup([]) => (Pat::Str(")"), Pat::Str("(")),
        TyKind::Tup([ty]) => ty_search_pat(ty),
        TyKind::Tup([head, .., tail]) => (ty_search_pat(head).0, ty_search_pat(tail).1),
        TyKind::OpaqueDef(..) => (Pat::Str("impl"), Pat::Str("")),
        TyKind::Path(qpath) => qpath_search_pat(&qpath),
        TyKind::Infer => (Pat::Str("_"), Pat::Str("_")),
        TyKind::TraitObject(_, _, TraitObjectSyntax::Dyn) => (Pat::Str("dyn"), Pat::Str("")),
        // NOTE: `TraitObject` is incomplete. It will always return true then.
        _ => (Pat::Str(""), Pat::Str("")),
    }
}

fn ident_search_pat(ident: Ident) -> (Pat, Pat) {
    (Pat::Sym(ident.name), Pat::Sym(ident.name))
}

pub trait WithSearchPat<'cx> {
    type Context: LintContext;
    fn search_pat(&self, cx: &Self::Context) -> (Pat, Pat);
    fn span(&self) -> Span;
}
macro_rules! impl_with_search_pat {
    (($cx_ident:ident: $cx_ty:ident<$cx_lt:lifetime>, $self:tt: $ty:ty) => $fn:ident($($args:tt)*)) => {
        impl<$cx_lt> WithSearchPat<$cx_lt> for $ty {
            type Context = $cx_ty<$cx_lt>;
            fn search_pat(&$self, $cx_ident: &Self::Context) -> (Pat, Pat) {
                $fn($($args)*)
            }
            fn span(&self) -> Span {
                self.span
            }
        }
    };
}
impl_with_search_pat!((cx: LateContext<'tcx>, self: Expr<'tcx>) => expr_search_pat(cx.tcx, self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: Item<'_>) => item_search_pat(self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: TraitItem<'_>) => trait_item_search_pat(self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: ImplItem<'_>) => impl_item_search_pat(self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: FieldDef<'_>) => field_def_search_pat(self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: Variant<'_>) => variant_search_pat(self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: Ty<'_>) => ty_search_pat(self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: Attribute) => attr_search_pat(self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: Ident) => ident_search_pat(*self));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: Lit) => lit_search_pat(&self.node));
impl_with_search_pat!((_cx: LateContext<'tcx>, self: Path<'_>) => path_search_pat(self));

impl<'cx> WithSearchPat<'cx> for (&FnKind<'cx>, &Body<'cx>, HirId, Span) {
    type Context = LateContext<'cx>;

    fn search_pat(&self, cx: &Self::Context) -> (Pat, Pat) {
        fn_kind_pat(cx.tcx, self.0, self.1, self.2)
    }

    fn span(&self) -> Span {
        self.3
    }
}

/// Checks if the item likely came from a proc-macro.
///
/// This should be called after `in_external_macro` and the initial pattern matching of the ast as
/// it is significantly slower than both of those.
pub fn is_from_proc_macro<'cx, T: WithSearchPat<'cx>>(cx: &T::Context, item: &T) -> bool {
    let (start_pat, end_pat) = item.search_pat(cx);
    !span_matches_pat(cx.sess(), item.span(), start_pat, end_pat)
}

/// Checks if the span actually refers to a match expression
pub fn is_span_match(cx: &impl LintContext, span: Span) -> bool {
    span_matches_pat(cx.sess(), span, Pat::Str("match"), Pat::Str("}"))
}

/// Checks if the span actually refers to an if expression
pub fn is_span_if(cx: &impl LintContext, span: Span) -> bool {
    span_matches_pat(cx.sess(), span, Pat::Str("if"), Pat::Str("}"))
}
