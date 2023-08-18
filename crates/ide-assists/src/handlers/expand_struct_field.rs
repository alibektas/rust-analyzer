use std::{ops::Deref, rc::Rc};

use crate::assist_context::{AssistContext, Assists};
use either::Either;
use hir::{
    db::ExpandDatabase, FieldSource, HasSource, HasVisibility, HirDisplay, InFile, PathResolution,
    SemanticsScope,
};
use ide_db::{
    assists::{AssistId, AssistKind},
    path_transform::{self, PathTransform},
};
use itertools::Itertools;
use syntax::{
    ast::{self, edit::IndentLevel, edit_in_place::HasVisibilityEdit, HasName},
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

    if let Some(hir::Adt::Struct(src_strukt)) = src_ty.as_adt() {
        let tgt_hir_strukt = ctx.sema.to_def(&tgt_strukt)?;
        let tgt_scope = ctx.sema.scope(&tgt_strukt.syntax())?;
        let tgt_module = tgt_hir_strukt.module(db);
        let tgt_field = tgt_field.clone_for_update();

        if !src_strukt.is_visible_from(db, tgt_module) {
            return None;
        }

        // if let Some(name) = tgt_scope.module().name(db) {
        //     eprintln!("Target scope name {}", name.to_smol_str());
        // }

        // if let Some(name) = src_scope.module().name(db) {
        //     eprintln!("Source scope name {}", name.to_smol_str());
        // }

        let flds = src_strukt
            .fields(db)
            .into_iter()
            .filter_map(|field| {
                if !field.is_visible_from(db, tgt_module) {
                    return None;
                }
                field.source(db)
            })
            .enumerate()
            .filter_map(|(idx, fld)| match fld.value {
                FieldSource::Named(n) => {
                    eprintln!("HEY BR");
                    let name = n.name()?.to_string();

                    dbg!(&name);
                    let ty = n.ty()?;
                    dbg!(&ty);

                    // eprintln!("Before transformation {}", ty.to_string());
                    // pt.apply(&ty.syntax());
                    // eprintln!("After transformation {}", ty.to_string());

                    Some(format!(
                        "{}_{} : {}",
                        tgt_field_name.to_string(),
                        name.to_string(),
                        ty.to_string()
                    ))
                }
                FieldSource::Pos(p) => {
                    let ty = p.ty()?;
                    Some(format!("{}_{} : {}", tgt_field_name.to_string(), idx, ty.to_string(),))
                }
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
                    flds.join(
                        format!(",\n{}", IndentLevel::from_node(tgt_field.syntax())).as_str(),
                    ),
                );
            },
        );
    }

    Some(())
}

#[cfg(test)]
mod tests {

    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn deneme() {
        check_assist(
            expand_struct_field,
            r#"
struct A {
    i : i32,
    j : i32,                
}

struct B {
    k$0 : A
}
"#,
            "",
        )
    }

    #[test]
    fn dep_is_generated_by_macro() {
        check_assist(
            expand_struct_field,
            r#"
macro_rules! create_struct {
    ($struct_name:ident , $($field_name:ident , $field_type:ty),*) => {
        pub struct $struct_name {
            $(
                $field_name: $field_type,
            )*
        }
    };
}
create_struct!(B, a, i32, b, i32);

struct A {
    i: i32,
    j$0: B,
}"#,
            r#""#,
        )
    }

    #[test]
    fn tgt_local_in_child_mod() {
        check_assist(
            expand_struct_field,
            r#"
mod K {

    pub struct C;
    struct D;

    pub(super) struct A {
        pub i: C,
        j: D,
    }
}

struct B {
    $0i: K::A,
}"#,
            r#""#,
        );
    }
}
