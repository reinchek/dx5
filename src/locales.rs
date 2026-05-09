use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use rocket_dyn_templates::tera;
use rocket_dyn_templates::tera::Function;
use serde_json::{to_value, Value};

fn flatten_json(value: &Value, prefix: &str, map: &mut HashMap<String, String>) {
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let new_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(v, &new_key, map);
            }
        }
        Value::String(s) => {
            map.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}

pub fn load_translations() -> HashMap<String, HashMap<String, String>> {
    let locales_dir = "config/locales";
    let mut all = HashMap::new();
    if let Ok(entries) = fs::read_dir(locales_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let lang = path.file_stem().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path).unwrap_or_default();
                let value: Value = serde_json::from_str(&content).unwrap_or_default();
                let mut map = HashMap::new();
                flatten_json(&value, "", &mut map);
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