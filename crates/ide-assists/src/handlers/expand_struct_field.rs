use std::collections::HashMap;

use crate::assist_context::{AssistContext, Assists};
use hir::{FieldSource, HasSource, HasVisibility, HirDisplay, StructKind};
use ide_db::assists::{AssistId, AssistKind};
use syntax::{
    ast::{
        self, edit::IndentLevel, GenericArg, HasGenericParams, HasName,
        HasVisibility as AstVisibility, LifetimeParam, RefType,
    },
    AstNode,
};

pub(crate) fn expand_struct_field(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let db = ctx.db();

    // Naming :
    // tgt stands for target
    // src stands for source
    // We want to spread/expand src in tgt

    // Find the node under cursor.
    // Is it a struct?
    let tgt_strukt = ctx.find_node_at_offset::<ast::Struct>()?;
    // Is it a record field? We allow this assist to be used
    // specifically for fields of this sort  otherwise it would just confuse the user
    // as it would be hard to determine which field belonged to the target or
    // the source struct.
    let tgt_field = ctx.find_node_at_offset::<ast::RecordField>()?;
    let tgt_field_name = tgt_field.name()?;

    // If field has a type already defined, resolve it to its source type.
    let src_ty = ctx.sema.resolve_type(&tgt_field.ty()?)?;

    let src_hir_strukt =
        if let Some(hir::Adt::Struct(skt)) = src_ty.as_adt() { skt } else { return None };

    // Adopt target field's visibility to the expanding fields of the source.
    let tgt_field_vis = if let Some(vis) = tgt_field.visibility() {
        format!("{} ", vis.to_string())
    } else {
        "".to_string()
    };

    // This assist should only be applicable to record structs.
    if !(matches!(src_hir_strukt.kind(db), StructKind::Record)) {
        return None;
    }

    let tgt_hir_strukt = ctx.sema.to_def(&tgt_strukt)?;
    let tgt_module = tgt_hir_strukt.module(db);
    let tgt_field = tgt_field.clone_for_update();
    // TODO let tgt_field_ty = tgt_field.ty()?;

    if !src_hir_strukt.is_visible_from(db, tgt_module) {
        return None;
    }

    let src_strukt = src_hir_strukt.source(db)?;

    let mut lifetime_map = HashMap::default();
    if let Some(a) = src_strukt.value.generic_param_list() {
        lifetime_map = tgt_field
            .ty()?
            .generic_arg_list()?
            .generic_args()
            .into_iter()
            .filter_map(|arg| {
                if let GenericArg::LifetimeArg(arg) = arg {
                    return Some(arg);
                } else {
                    return None;
                }
            })
            .zip(a.lifetime_params().collect::<Vec<LifetimeParam>>())
            .collect::<HashMap<ast::LifetimeArg, ast::LifetimeParam>>();
    }

    let flds = src_hir_strukt
        .fields(db)
        .into_iter()
        .filter(|field| {
            if !field.is_visible_from(db, tgt_module) {
                return false;
            }

            true
        })
        .filter_map(|fld| {
            if let Some(source_field) = fld.source(db) {
                let field_ast = source_field.value;
                if let FieldSource::Named(field_ast) = field_ast {
                    dbg!("ABC", &field_ast.to_string());
                    let ty = field_ast.ty()?;
                    dbg!(&ty.to_string());

                    if let ast::Type::RefType(rf) = ty {
                        dbg!(rf.lifetime());
                    }

                    // arg_list.lifetime_args().map(|arg| {
                    //     dbg!(&arg , lifetime_map.get(&arg));
                    // });
                }
            }

            Some(format!(
                "{}{}_{} : {}",
                tgt_field_vis,
                tgt_field_name.to_string(),
                fld.name(db).as_text()?,
                "TODO"
            ))
        })
        .collect::<Vec<String>>();

    if flds.is_empty() {
        eprintln!("Field count is zero. Early exit.");
        return None;
    }

    return acc.add(
        AssistId("expand_struct_field", AssistKind::Generate),
        "Expand struct field",
        tgt_field_name.syntax().text_range(),
        |edit| {
            edit.replace(
                tgt_field.syntax().text_range(),
                flds.join(format!(",\n{}", IndentLevel::from_node(tgt_field.syntax())).as_str()),
            );
        },
    );
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn src_in_same_mod() {
        check_assist(
            expand_struct_field,
            r#"
struct A {
    i : i32,
    j : i32,
}

struct B {
    k$0 : A
}"#,
            r#"
struct A {
    i : i32,
    j : i32,
}

struct B {
    k_i : i32,
    k_j : i32
}"#,
        )
    }

    #[test]
    fn src_not_vis() {
        check_assist_not_applicable(
            expand_struct_field,
            r#"
mod k {
    struct Source {
        x: i32,
        y: i32,
    }
}

struct Target {
    pub x$0y : k::Source,
    pub z : i32,
}"#,
        )
    }

    #[test]
    fn src_in_sibling_mod() {
        check_assist(
            expand_struct_field,
            r#"
mod k {
    pub(super) struct Source {
        pub x: i32,
        pub y: i32,
    }
}

struct Target {
    pub x$0y : k::Source,
    pub z : i32,
}"#,
            r#"
mod k {
    pub(super) struct Source {
        pub x: i32,
        pub y: i32,
    }
}

struct Target {
    pub xy_x : i32,
    pub xy_y : i32,
    pub z : i32,
}"#,
        );
    }

    #[test]
    fn src_in_sibling_mod_no_vis_fields() {
        check_assist_not_applicable(
            expand_struct_field,
            r#"
mod k {
    pub(super) struct Source {
        x: i32,
        y: i32,
    }
}

struct Target {
    pub x$0y : k::Source,
    pub z : i32,
}"#,
        )
    }

    #[test]
    fn src_in_sibling_mod_some_vis_fields() {
        check_assist(
            expand_struct_field,
            r#"
mod k {
    pub(super) struct Source {
        pub(super) x: i32,
        y: i32,
    }
}

struct Target {
    pub x$0y : k::Source,
    pub z : i32,
}"#,
            r#"
mod k {
    pub(super) struct Source {
        pub(super) x: i32,
        y: i32,
    }
}

struct Target {
    pub xy_x : i32,
    pub z : i32,
}"#,
        )
    }

    #[test]
    fn src_has_lifetimes() {
        check_assist(
            expand_struct_field,
            r#"
struct Source<'a , 'b> {
    a : &'a str,
    b : &'b str
}

struct Target<'a> {
    sr$0c : Source<'a, 'a>
}
    "#,
            r#"
struct Source<'a , 'b> {
    a : &'a str,
    b : &'b str
}

struct Target<'a> {
    src_a : &'a str,
    src_b : &'a str
}
"#,
        )
    }

    #[test]
    fn test_1() {
        check_assist(
            expand_struct_field,
            r#"
struct Source<'a, D> {
    i: C<'a, D>,
    j: i32,
}

struct C<'abc, D> {
    k: &'abc D,
}

struct Target<'def> {
    a: Sour$0ce<'def, i32>,
}"#,
            r#"
struct Source<'a, D> {
    i: C<'a, D>,
    j: i32,
}

struct C<'abc, D> {
    k: &'abc D,
}

struct Target<'def> {
    a_i: C<'def, i32>,
    a_j: i32,
}
"#,
        )
    }

    #[test]
    fn test_2() {
        check_assist(
            expand_struct_field,
            r#"
struct Source<T>
where
    T: Iterator,
    T::Item: Copy,
    String: PartialEq<T>,
    i32: Default,
{
    f: T,
}

struct Target<T>
where
    T: Iterator,
    T::Item: Copy,
    String: PartialEq<T>,
    i32: Default,
{
    a: Sou$0rce<T>,
}            
            "#,
            r#"
struct Source<T>
where
    T: Iterator,
    T::Item: Copy,
    String: PartialEq<T>,
    i32: Default,
{
    f: T,
}

struct Target<T>
where
    T: Iterator,
    T::Item: Copy,
    String: PartialEq<T>,
    i32: Default,
{
    a_f: T,
}"#,
        )
    }

    #[test]
    fn test_3() {
        check_assist(
            expand_struct_field,
            r#"
struct Source<T, const N: usize> {
    a: [T; N],
}

struct Target {
    b: So$0urce<i32, 5>,
}
            "#,
            r#"
struct Source<T, const N: usize> {
    a: [T; N],
}

struct Target {
    b_a: [i32; 5],
}
"#,
        )
    }

    #[test]
    fn test_4() {
        check_assist(
            expand_struct_field,
            r#"
struct Source<T, const N: usize = 5> {
    a: [T; N],
}

struct Target {
    b: So$0urce<i32>,
}

"#,
        r#"
struct Source<T, const N: usize = 5> {
    a: [T; N],
}

struct Target {
    b: [i32; 5],
}"#
        )
    }

    #[test]
    fn test_5() {
        check_assist(
            expand_struct_field,
            r#"
struct Source<T = i32> {
    b: T,
}

struct Target {
    a: So$0urce,
}
"#,
            r#"
struct Source<T = i32> {
    b: T,
}

struct Target {
    a_b: i32,
}
    "#
        )
    }

    #[test]
    fn test_6() {
        check_assist(
            expand_struct_field,
            r#"
struct Source<T = i32> {
    b: T,
}

struct Target {
    a: So$0urce,
}
"#,
            r#"
struct Source<T = i32> {
    b: T,
}

struct Target {
    a_b: i32,
}
"#
        )
    }
}
