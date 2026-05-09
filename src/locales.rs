use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use rocket_dyn_templates::tera;
use rocket_dyn_templates::tera::Function;
use serde_json::{to_value, Value};

pub fn load_translations() -> HashMap<String, HashMap<String, String>> {
    let locales_dir = "config/locales";
    let mut all = HashMap::new();
    if let Ok(entries) = fs::read_dir(locales_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let lang = path.file_stem().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path).unwrap_or_default();
                let map: HashMap<String, String> = serde_json::from_str(&content).unwrap_or_default();
                all.insert(lang, map);
            }
        }
    }
    all
}

pub fn make_translator(
    translations: Arc<HashMap<String, HashMap<String, String>>>,  // <lang, <key, value>>
) -> impl Function {
    move |args: &HashMap<String, Value>| -> tera::Result<Value> {
        let key = args
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("Missing 'key' argument"))?;

        let lang = args
            .get("lang")
            .and_then(|v| v.as_str())
            .unwrap_or("en");

        let translation = translations
            .get(lang)
            .and_then(|m| m.get(key))
            .map(|s| s.as_str())
            .unwrap_or(key);

        Ok(to_value(translation).unwrap())
    }
}