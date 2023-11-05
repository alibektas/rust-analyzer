use super::flags;
use ide_db::{base_db::SourceDatabase, RootDatabase};
use itertools::Itertools;

impl flags::Oguz {
    pub fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use hir::{HirDisplay, Semantics};
    use load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};

    use super::canon_path;

    fn test_script() -> Option<()> {
        let no_progress = &|s| (eprintln!("rust-analyzer: Loading {s}"));
        let load_cargo_config = LoadCargoConfig {
            load_out_dirs_from_check: true,
            with_proc_macro_server: ProcMacroServerChoice::Sysroot,
            prefill_caches: true,
        };
        let root = vfs::AbsPathBuf::assert(PathBuf::from("/path/to/project")).normalize();

        let config = crate::config::Config::new(
            root.clone(),
            lsp_types::ClientCapabilities::default(),
            /* workspace_roots = */ vec![],
            /* is_visual_studio_code = */ false,
        );

        let cargo_config = config.cargo();
        let (host, vfs, _) = load_workspace_at(
            root.as_path().as_ref(),
            &cargo_config,
            &load_cargo_config,
            &no_progress,
        )
        .unwrap();

        let db = host.raw_database();
        let _analysis = host.analysis();

        // Buraya kadar seni ilgilendiren bi kisim yok
        // rust analyzer projeyi tariyor ilgili yerleri seciyor
        // db'ni atiyor butun analizini yapiyor.
        // Burasi StaticIndex de kullanabilirdi ama hic karsima cikmadi
        // ne ise yarar bilmem ilgili ornekleri SCIP'de bulabilirsin.

        // Bir baslangic file'i vermek gerekiyor ki bahsettigin recursion
        // baslasin.
        let file_start = vfs::VfsPath::from(
            vfs::AbsPathBuf::assert(PathBuf::from("/abs/path/to/main.rs")).normalize(),
        );

        // Baslangic file'i
        let root_file_id = vfs.file_id(&file_start)?;
        // Henuz bir onemi olmasa da belki bi ise yarar diye dusundugum bir obje.
        // Ama sanirim bu olmadan da yolumuzu bulabiliyoruz.
        let sema = Semantics::new(db);

        // Root_file'a ait modulu getir.
        let module = sema.to_module_def(root_file_id)?;
        // Neler tanimli burada onu getir.
        let items = module.scope(sema.db, None);

        for (_name, def) in items {
            match def {
                hir::ScopeDef::ModuleDef(mdef) => match mdef {
                    hir::ModuleDef::Module(_) => (),
                    hir::ModuleDef::Function(_) => (),
                    hir::ModuleDef::Adt(adt) => match adt {
                        hir::Adt::Struct(strukt) => {
                            dbg!(canon_path(
                                db,
                                &module,
                                Some(strukt.name(db).to_smol_str().to_string())
                            ));
                            for field in strukt.fields(db) {
                                dbg!(&field
                                    .ty(db)
                                    .display_source_code(db, module.into(), false)
                                    .ok()?);
                            }
                        }
                        hir::Adt::Union(_) => (),
                        hir::Adt::Enum(_) => (),
                    },
                    hir::ModuleDef::Variant(_) => (),
                    hir::ModuleDef::Const(_) => (),
                    hir::ModuleDef::Static(_) => (),
                    hir::ModuleDef::Trait(_) => (),
                    hir::ModuleDef::TraitAlias(_) => (),
                    hir::ModuleDef::TypeAlias(_) => (),
                    hir::ModuleDef::BuiltinType(_) => (),
                    hir::ModuleDef::Macro(_) => (),
                },
                hir::ScopeDef::AdtSelfType(adt) => match adt {
                    _ => (),
                },
                a => _ = dbg!(&a),
            }
        }

        Some(())
    }

    #[test]
    fn test_1() {
        test_script().unwrap()
    }
}

// Rust analyzer'da gerekmedikce canonical path olusturma gibi bir ihtiyac yok
// zaten database'inde path tutma diye bi sey hic yok, dolayisyla gerektikce kendin olusturuyorsun.
pub fn canon_path(db: &RootDatabase, module: &hir::Module, item_name: Option<String>) -> String {
    let crate_name =
        db.crate_graph()[module.krate().into()].display_name.as_ref().map(|it| it.to_string());
    let module_path = module
        .path_to_root(db)
        .into_iter()
        .rev()
        .flat_map(|it| it.name(db).map(|name| name.display(db).to_string()));
    crate_name.into_iter().chain(module_path).chain(item_name).join("::")
}
