use std::collections::HashMap;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MenuItem {
    pub icon:  Option<String>,
    pub key:   String,
    pub path:  String, // @todo: HasMap<language, path_per_lang>
    pub label: String,
    pub order: u8
}