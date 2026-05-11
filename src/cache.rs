use crate::contents::ContentTypeDef;
use crate::errors::Dx5Error;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::fs::read_dir;
use std::path::Path;

#[derive(Deserialize)]
struct FileCreated {
    created: Option<String>,
}

fn get_created(path: &Path) -> String {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| {
            Matter::<YAML>::new()
                .parse::<FileCreated>(&raw)
                .ok()?
                .data?
                .created
        })
        .unwrap_or_default()
}

// TODO: Rename in fake-cache.rs
pub fn split_contents_in_pages_folders(
    languages: Vec<&String>,
    content_type_def: &ContentTypeDef,
    rebuild: bool,
) -> Result<(), Dx5Error> {
    let dir = &content_type_def.dir;

    for language in languages {
        let dir = format!("{}/{}", dir, language);
        let pages_hidden_dir = format!("{}/{}", dir, ".pages");

        // If rebuild is true, remove recursive .pages folder.
        if rebuild && Path::new(&pages_hidden_dir).exists() {
            fs::remove_dir_all(&pages_hidden_dir)?;
        }

        // Create (if not exists) the .pages folder
        fs::create_dir_all(&pages_hidden_dir).map_err(|e| Dx5Error::io(e.to_string()))?;

        // Get all files in <dir> (excluding .pages)
         let files_in_dir = read_dir(&dir)
            .map_err(|e| Dx5Error::io(e.to_string()))?;

        // Filter out only YAML files ;)
        let mut files_in_dir = files_in_dir
            .flatten()
            .filter(|file| file
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("") == "yaml")
            .collect::<Vec<_>>();

        // Sort by `created` front matter field
        files_in_dir.sort_by(|a, b| {
            let ordering = &content_type_def.ordering;
            let a_created = get_created(&a.path());
            let b_created = get_created(&b.path());
            if ordering == "ASC" {
                a_created.cmp(&b_created)
            } else {
                b_created.cmp(&a_created)
            }
        });

        // Divide files by <pagination_items_per_page>.
        let chunks = files_in_dir.chunks(
            content_type_def.pagination_items_per_page.unwrap_or(10) as usize
        );
        for (page_num, page_files) in chunks.enumerate() {
            let page_num = page_num + 1;
            let dest = format!("{}/{}", pages_hidden_dir, page_num);

            fs::create_dir_all(&dest).expect("[FS] Error during folder creation.");

            for file in page_files {
                let file_name = file.file_name().to_string_lossy().to_string();
                let dest_file = format!("{}/{}", dest, file_name);
                fs::copy(file.path(), Path::new(&dest_file)).ok();
            }
        }
    }

    Ok(())
}

pub fn find_content_file_in_pages(dir: &Path, file_name: &str) -> Result<String, Dx5Error> {
    let mut result = String::new();

    let Ok(entries) = fs::read_dir(dir) else { return Ok(result); };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            return find_content_file_in_pages(path.as_path(), file_name)
        } else if path.file_name().unwrap().to_str().unwrap() == file_name {
            result = path.display().to_string();
            return Ok(result);
        }
    }

    Ok(result)
}
