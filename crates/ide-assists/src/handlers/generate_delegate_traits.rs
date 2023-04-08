use std::collections::HashSet;

use hir::{self, HasCrate, HasSource, HasVisibility};
use syntax::ast::{self, make, AstNode, HasGenericParams, HasName, HasVisibility as _};

use crate::{
    utils::{convert_param_list_to_arg_list, find_struct_impl, render_snippet, Cursor},
    AssistContext, AssistId, AssistKind, Assists, GroupLabel,
};
use syntax::ast::edit::AstNodeEdit;

pub(crate) fn generate_delegate_traits(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let strukt = ctx.find_node_at_offset::<ast::Struct>()?;
    let strukt_name = strukt.name()?;
    let current_module = ctx.sema.scope(strukt.syntax())?.module();
    let (field_name, field_ty, target) = match ctx.find_node_at_offset::<ast::RecordField>() {
        Some(field) => {
            let field_name = field.name()?;
            let field_ty = field.ty()?;
            (field_name.to_string(), field_ty, field.syntax().text_range())
        }
        None => {
            let field = ctx.find_node_at_offset::<ast::TupleField>()?;
            let field_list = ctx.find_node_at_offset::<ast::TupleFieldList>()?;
            let field_list_index = field_list.fields().position(|it| it == field)?;
            let field_ty = field.ty()?;
            (field_list_index.to_string(), field_ty, field.syntax().text_range())
        }
    };

    dbg!(&field_name , &field_ty , &target);
    
    // acc.add(
    //     AssistId("generate_delegate_traits" , AssistKind::Generate) , 
    //     format!("Generate delegate for `{field_name}.{}()`"),
    //     target,
    //     | builder |  {
            
    //     }
    // );

    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn test_generate_delegate_trait_impl() {
        check_assist(
            generate_delegate_trait,
            r#"
struct Struct {
    field$0: i32,
}

trait Trait {
    fn foo(&self) -> i32;
}

impl Trait for i32 {
    fn foo(&self) -> i32 {
        *self
    }
}

"#,
            r#"
struct Struct {
    field: i32,
}

trait Trait {
    fn foo(&self) -> i32;
}

impl Trait for i32 {
    fn foo(&self) -> i32 {
        *self
    }
}

impl Trait for Struct {
    fn foo(&self) -> i32 {
        self.field.foo()
    }
}
"#,
        )
    }
}
