use crate::config::Config as Dx5Config;
use crate::contents::{ContentTypeDef, ContentTypesConfig};
use notify::event::CreateKind;
use notify::event::DataChange;
use notify::event::ModifyKind::{Data, Name};
use notify::event::RemoveKind;
use notify::event::RenameMode;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::canonicalize;
use std::sync::mpsc::channel;

pub fn start_filesystem_watcher(dx5_config: &Dx5Config) -> Result<(), notify::Error> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    let content_types = ContentTypesConfig::init();
    let mut paths = vec![];

    // Register all directories to watch.
    for (_type_name, def) in content_types.types.iter() {
        for lang in &dx5_config.languages.0 {
            let path = format!("{}/{}", def.dir, lang.0);
            paths.push(path);
        }
    }

    for p in paths {
        println!("[FS][info] Watching {}", p);
        match canonicalize(&p) {
            Ok(abs) => {
                println!("[FS][info] {}", abs.display());
                watcher.watch(&abs, RecursiveMode::NonRecursive)?;
            }
            Err(e) => {
                eprintln!("[FS][warn] Skipping '{}': {}", p, e);
            }
        }
    }

    std::thread::spawn(move || {
        let _watcher = watcher;

        let rebuild_pages = |file_path: &str| {
            let config = crate::config::Config::load().unwrap();
            let languages = config.languages.0.keys().collect();
            let type_def = ContentTypeDef::from_path(&file_path.to_string()).unwrap();
            crate::cache::split_contents_in_pages_folders(languages, &type_def, true).unwrap();
        };

        let _sync_to_cache = |file_path: &str, file_name: &str| {
            let dest_dir = file_path.replace(file_name, ".pages");
            if let Ok(dest_path) =
                crate::cache::find_content_file_in_pages(std::path::Path::new(&dest_dir), file_name)
            {
                std::fs::copy(file_path, dest_path).ok();
            }
        };

        for res in rx {
            match res {
                Ok(event) => {
                    let is_yaml = |p: &std::path::PathBuf| -> bool {
                        p.extension().is_some_and(|e| e == "yaml")
                    };

                    if !event.paths.iter().any(&is_yaml) {
                        continue;
                    }

                    match event.kind {
                        // File created, deleted, or renamed (*and modified) - rebuild .pages entirely
                        EventKind::Modify(Data(DataChange::Any)) |
                        EventKind::Create(CreateKind::File) |
                        EventKind::Remove(RemoveKind::File) |
                        EventKind::Modify(Name(RenameMode::From)) |
                        EventKind::Modify(Name(RenameMode::To)) => {
                            if let Some(p) = event.paths.first().filter(|p| is_yaml(p)) {
                                let file_path = p.to_string_lossy().to_string();
                                eprintln!("[FS][info] Rebuilding .pages for {}", file_path);
                                rebuild_pages(&file_path);
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => eprintln!("[FS] watch error: {:?}", e),
            }
        }
    });

    Ok(())
}
