use crate::assist_context::{AssistContext, Assists};
use hir::{HasSource, HasVisibility, HirDisplay};
use ide_db::assists::{AssistId, AssistKind};
use syntax::{
    ast::{self, edit::IndentLevel, HasName, HasVisibility as AstVisibility, Lifetime, RefType},
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
        let tgt_field_vis = if let Some(vis) = tgt_field.visibility() {
            format!("{} ", vis.to_string())
        } else {
            "".to_string()
        };

        let src_strukt_kind = match src_strukt.kind(db) {
            hir::StructKind::Unit => return None,
            kind => kind,
        };
        let tgt_hir_strukt = ctx.sema.to_def(&tgt_strukt)?;
        let tgt_module = tgt_hir_strukt.module(db);
        let tgt_field = tgt_field.clone_for_update();
        let tgt_field_ty = tgt_field.ty()?;

        #[rustfmt::skip]
        let ref_prefix: TypeKind  = 
            if let ast::Type::RefType(rf) = tgt_field_ty {
                if let Some(mut_token) = rf.mut_token() {
                    TypeKind::Mut(RefTy { lifetime: rf.lifetime()?.to_string() })
                } else {
                    TypeKind::Shared(RefTy { lifetime : rf.lifetime()?.to_string()})
                }
            } else {
                TypeKind::Owned
            };

        

        if !src_strukt.is_visible_from(db, tgt_module) {
            return None;
        }

        let flds = src_strukt
            .fields(db)
            .into_iter()
            .filter_map(|field| {
                if !field.is_visible_from(db, tgt_module) {
                    return None;
                }

                Some(field)
            })
            .filter_map(|fld| {
                let targeted_ty =
                    if let Ok(tty) = fld.ty(db).display_source_code(db, tgt_module.into(), true) {
                        tty
                    } else {
                        return None;
                    };

                Some(format!(
                    "{}{}_{} :{}",
                    tgt_field_vis,
                    if let hir::StructKind::Record = src_strukt_kind {
                        tgt_field_name.to_string()
                    } else {
                        tgt_field_name.to_string()
                    },
                    fld.name(db).as_text()?,
                    targeted_ty
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
                    flds.join(
                        format!(",\n{}", IndentLevel::from_node(tgt_field.syntax())).as_str(),
                    ),
                );
            },
        );
    }

    Some(())
}


enum TypeKind {
    Mut(RefTy),
    Shared(RefTy),
    Owned
} 

struct RefTy {
    lifetime : String,
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
            "
struct A {
    i : i32,
    j : i32,
}

struct B {
    k_i : i32,
    k_j : i32
}",
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
}"#,
            r#"
struct Source<'a , 'b> {
    a : &'a str,
    b : &'b str
}

struct Target<'a> {
    src_a : &'a str,
    src_b : &'a str
}"#,
        )
    }
}
