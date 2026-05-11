use crate::config::ConfigError;
use crate::errors::Dx5Error;
use crate::menu_item::MenuItem;
use crate::AppConfig;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::{read_dir, read_to_string};
use std::path::Path;

pub const TEMPLATE_FILENAME: &str = "0x00.template";
pub const DEFAULT_TYPES_TEMPLATE_FOLDER: &str = "types";

#[derive(Serialize, Debug)]
pub struct PaginationObject {
    pub current: usize,
    pub per_page: usize,
    pub pages: Vec<usize>,
    pub total_pages: usize,
    pub total_items: usize,
    pub has_prev: bool,
    pub has_next: bool,
    pub ordering: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BodyFieldDyn {
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(flatten)]
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub id: String,
    // pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created: String,
    pub updated: Option<String>,
    pub content: Option<String>,
    pub body_fields: Vec<BodyFieldDyn>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentIndex {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created: String,
    pub updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentTypeDef {
    pub dir: String,
    #[serde(default = "ContentTypeDef::default_extension")]
    pub extension: String,
    pub route: String,
    pub template: Option<String>,
    pub menu_icon: Option<String>,
    pub menu_label: String,
    pub menu_order: u8,
    pub is_default: Option<bool>,
    #[serde(default = "ContentTypeDef::default_ordering")]
    pub ordering: String,
    pub pagination_items_per_page: Option<u8>,
    pub enable_navigation: Option<bool>,
}

impl ContentTypeDef {
    pub fn from_path(path_to_search: &String) -> Result<ContentTypeDef, ConfigError> {
        for (_type_name, type_def) in ContentTypesConfig::init().types.iter() {
            let path = Path::new(&path_to_search);
            let type_def_dir = fs::canonicalize(Path::new(&type_def.dir)).unwrap();
            if path.starts_with(type_def_dir) {
                return Ok(type_def.to_owned());
            }
        }

        Err(ConfigError("No matching results type found.".to_string()))
    }

    pub fn default_extension() -> String {
        String::from("yaml")
    }
    pub fn default_ordering() -> String {
        String::from("DESC")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentTypesConfig {
    pub types: HashMap<String, ContentTypeDef>,
}

impl ContentTypesConfig {
    pub fn init() -> Self {
        let path = "config/dx5.content_types.toml";
        let raw = fs::read_to_string(path).map_err(|_| {
            ConfigError(format!(
                "Configuration file '{}' not found.\nCreate a dx5.content_types.toml file in the project's root.",
                path
            ))
        }).unwrap();

        let ctypes_cfg: ContentTypesConfig = toml::from_str(&raw)
            .map_err(|e| {
                ConfigError(format!(
                    "Error during dx5.content_types.toml file parsing: {}",
                    e
                ))
            })
            .unwrap();

        ctypes_cfg
    }

    pub fn get_default(&self) -> Option<(&String, &ContentTypeDef)> {
        self.types.iter().find_map(|(name, type_def)| {
            if type_def.is_default == Some(true) {
                Some((name, type_def))
            } else {
                None
            }
        })
    }

    pub fn get_enable_navigation(&self, type_name: &String) -> Option<bool> {
        self.types.get(type_name).and_then(|def| {
            if def.enable_navigation == Some(true) {
                Some(true)
            } else {
                None
            }
        })
    }

    pub fn as_menu_items(&self) -> Vec<MenuItem> {
        let mut items: Vec<_> = self
            .types
            .iter()
            .map(|(key, def)| MenuItem {
                icon: def.menu_icon.clone(),
                key: key.clone(),
                label: def.menu_label.clone(),
                path: format!("/{}", key),
                order: def.menu_order,
            })
            .collect();
        items.sort_by_key(|i| i.order);
        items
    }
}

impl Content {
    pub fn load(
        lang: String,
        id: &str,
        matter: &Matter<YAML>,
        cfg: &ContentTypeDef,
    ) -> Result<Self, Dx5Error> {
        let path = format!("{}/{}/{}.{}", cfg.dir, lang, id, cfg.extension);
        let raw = read_to_string(&path).map_err(|_| Dx5Error::not_found(id.to_string()))?;

        let parsed = matter
            .parse::<Self>(&raw)
            .map_err(|e| Dx5Error::parse(e.to_string()))?;

        let mut content = parsed
            .data
            .ok_or_else(|| Dx5Error::parse(format!("Missing front matter in '{}'", id)))?;

        content.content = Some(parsed.content);

        Ok(content)
    }

    pub fn load_home(
        lang: Option<String>,
        app_config: &AppConfig,
        matter: &Matter<YAML>,
        cfg: &ContentTypeDef,
    ) -> Result<Self, Dx5Error> {
        let lang = if let Some(l) = lang {
            l
        } else {
            app_config.0.blog.language.clone()
        };

        let main_path_part = cfg.dir.replace("./", "");
        let main_path_part = main_path_part.split("/").collect::<Vec<&str>>()[0];
        let path = format!("./{}/home/home.{}.yaml", main_path_part, lang);
        let raw = read_to_string(path).map_err(|_| Dx5Error::not_found("home".to_string()))?;

        let parsed = matter
            .parse::<Self>(&raw)
            .map_err(|e| Dx5Error::parse(e.to_string()))?;

        let mut content = parsed
            .data
            .ok_or_else(|| Dx5Error::parse("Missing front matter in 'home.yaml'".to_string()))?;

        content.content = Some(parsed.content);

        Ok(content)
    }

    pub fn all(
        lang: &String,
        matter: &Matter<YAML>,
        cfg: &ContentTypeDef,
        page: Option<usize>,
    ) -> Result<(Vec<ContentIndex>, Option<PaginationObject>), Dx5Error> {
        // let mut entries = Self::get_index(&lang, matter, cfg)?;
        let pages_path = format!("{}/{}/.pages", &cfg.dir, lang);

        let mut pages = read_dir(&pages_path)
            .map_err(|_| Dx5Error::not_found("pages".to_string()))?
            .filter(|d| {
                d.as_ref()
                    .map(|entry| entry.path().is_dir())
                    .unwrap_or(false)
            })
            .map(|d| {
                d.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .to_string()
                    .parse::<usize>()
            })
            .collect::<Result<Vec<usize>, _>>()
            .map_err(|_| Dx5Error::not_found("pages".to_string()))?
            .into_iter()
            .collect::<Vec<usize>>();
        pages.sort();

        let page_path = format!("{}/{}", &pages_path, &page.unwrap_or(1));

        let entries = read_dir(&page_path)
            .map_err(|e| Dx5Error::io(format!("Unable to read '{}': {}", cfg.dir, e)))?;

        let mut contents = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            match ContentIndex::load(lang, stem, matter, cfg) {
                Ok(item) => contents.push(item),
                Err(e) => eprintln!("[WARN] Skipping '{}': {}", stem, e),
            }
        }

        let page = page.unwrap_or(1);

        let ordering = &cfg.ordering;
        if ordering == "ASC" {
            contents.sort_by(|a, b| a.created.cmp(&b.created));
        } else {
            contents.sort_by(|a, b| b.created.cmp(&a.created));
        }

        let pagination_object = Some(PaginationObject {
            current: page,
            per_page: cfg.pagination_items_per_page.unwrap_or(10) as usize,
            total_pages: pages.len(),
            pages: pages.clone(),
            total_items: contents.len(),
            ordering: cfg.ordering.clone(),
            has_prev: page > 1,
            has_next: page < pages.len(),
        });

        Ok((contents, pagination_object))
    }
}

impl ContentIndex {
    pub fn load(
        lang: &String,
        id: &str,
        matter: &Matter<YAML>,
        cfg: &ContentTypeDef,
    ) -> Result<Self, Dx5Error> {
        let path = format!("{}/{}/{}.{}", cfg.dir, lang, id, cfg.extension);
        let raw = read_to_string(&path).map_err(|_| Dx5Error::not_found(id.to_string()))?;

        let parsed = matter
            .parse::<Self>(&raw)
            .map_err(|e| Dx5Error::parse(e.to_string()))?;

        let content = parsed
            .data
            .ok_or_else(|| Dx5Error::parse(format!("Missing front matter in '{}'", id)))?;

        Ok(content)
    }
}
