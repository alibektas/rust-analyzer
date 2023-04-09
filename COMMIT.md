# Notes for self

Here's a quick map of what needs to be done.
This is the relevant [issue](https://github.com/rust-lang/rust-analyzer/issues/14386).
It says that there is already an implementation for generating `Deref` for a field
and this can be generalized to `Delegated trait implementation`


Let's first see what `Deref` is all about.

## Deref 

Generate `Deref` impl using the given struct field.

```rust
# //- minicore: deref, deref_mut
struct A;
struct B {
   $0a: A
}
```
  
```rust
struct A;
struct B {
   a: A
}
impl core::ops::Deref for B {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.a
    }
}
```

Generically `Deref` is defined as

```rust 
pub trait Deref {
    type Target: ?Sized;

    fn deref(&self) -> &Self::Target;
}
```

## How does it relate to what we are doing?

This is just one possible delegated trait implementation.
As further examples we are given 

```rust
pub(crate) struct RibStack<R> {
    $0ribs: Vec<R>,
    used: usize,
}  
```

```rust
impl<R> std::ops::Deref for RibStack<R> {
    type Target = <Vec<R> as std::ops::Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.ribs // note that for deref we should special case emitting `&self.field` instead of `self.field.deref()`/`Deref::deref(&self.field)`
    }
}

impl<R> std::ops::DerefMut for RibStack<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ribs // note that for deref we should special case emitting `&mut self.field` instead of `self.field.deref_mut()`/`DerefMut::deref_mut(&mut self.field)`
    }
}

impl<R> IntoIterator for RibStack<R> {
    type Item = <Vec<R> as IntoIterator>::Item;
    type IntoIter = <Vec<R> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.ribs)
    }
}
```

Examples are from the [said issue](https://github.com/rust-lang/rust-analyzer/issues/14386).
One sentence is here of importance.
It states that the newly defined assist "should offer delegating traits impls implemented by `Vec<R>` (because that's what `ribs` has as its type)"

## Inquiries 

### How to locate where the cursor is at ?

```rust
fn generate_record_deref(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // By using the turbo-fish we kind of say that we want a struct to be found. 
    let strukt = ctx.find_node_at_offset::<ast::Struct>()?;
    let field = ctx.find_node_at_offset::<ast::RecordFieldl()?;
    //...
```

### How to check if a trait is implemented for a type?

An example comes from [here](./crates/ide-assists/src/handlers/generate_deref.rs#L150).

```rust
 fn existing_deref_impl(
    sema: &hir::Semantics<'_, RootDatabase>,
    strukt: &ast::Struct,
) -> Option<DerefType> {
    let strukt = sema.to_def(strukt)?;
    let krate = strukt.module(sema.db).krate();

    let deref_trait = FamousDefs(sema, krate).core_ops_Deref()?;
    let deref_mut_trait = FamousDefs(sema, krate).core_ops_DerefMut()?;
    let strukt_type = strukt.ty(sema.db);

    if strukt_type.impls_trait(sema.db, deref_trait, &[]) {
        if strukt_type.impls_trait(sema.db, deref_mut_trait, &[]) {
            Some(DerefType::DerefMut)
        } else {
            Some(DerefType::Deref)
        }
    } else {
        None
    }
}   
```
[This](crates/ide-db/src/famous_defs.rs) module will be important at some point.
Same goes for `Struct::impls_trait`

### How to generate code ?

As far as I can tell there are two different approaches each of which are used resp. in [here](./crates/ide-assists/src/handlers/generate_deref.rs#L122) and [here](./crates/ide-assists/src/handlers/generate_delegate_methods.rs#L95)
 
## Steps

1. Q : 
2. How to get the related data type?
3. How to get what the data type implements?
    1. QUESTION : Should we fetch all the traits that Vec<R> and its parent types implement?  


## Exodus

- You should write at least half a dozen of tests. 
- You should record a GIF
