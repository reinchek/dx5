use std::collections::HashMap;
use rocket_dyn_templates::tera;
use rocket_dyn_templates::tera::Function;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::ConfigError;
use crate::state::Globals;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldDefinition {
    pub template: Option<String>,
    pub description: Option<String>,
    pub schema: HashMap<String, String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldsConfig {
    pub fields: HashMap<String, FieldDefinition>,
}

impl FieldsConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let fields_in_toml = include_str!("../config/dx5.fields.toml");

        toml::from_str(fields_in_toml).map_err(|e| ConfigError(e.to_string()))
    }
}

//---------------------------------------------------------------------------
// Custom fields inclusion, based on dx5.fields.toml+Tera custom function
//---------------------------------------------------------------------------
pub fn make_fields_renderer(
    fields_config: FieldsConfig,
    globals: Globals,
    tera_snapshot: tera::Tera,
) -> impl Function {
    move |args: &HashMap<String, Value>| -> tera::Result<Value> {
        let field_val = args
            .get("field")
            .ok_or_else(|| tera::Error::msg("render_field: 'field' argument is missing"))?;

        let field_type = args.get("type").and_then(|t| t.as_str()).ok_or_else(|| {
            tera::Error::msg("render_field: 'type' argument is missing or not a string")
        })?;

        let def = fields_config.fields.get(field_type).ok_or_else(|| {
            tera::Error::msg(format!(
                "render_field: type '{}' not exists in dx5.fields.toml",
                field_type
            ))
        })?;

        let template_name = def
            .template
            .clone()
            .unwrap_or_else(|| format!("components/fields/{}", field_type));

        let mut ctx = tera::Context::new();

        // Check if field has safe = true (otherwise is false).
        let safe = field_val.get("safe").and_then(|v| v.as_bool()).unwrap_or(false);
        // If safe_prop isn't set use value prop as default one.
        let safe_prop = field_val.get("safe_prop").and_then(|v| v.as_str()).unwrap_or("value");

        // Clone field_val to replace the safe_prop prop with already processed one.
        let mut processed_field = field_val.clone();

        if let Some(raw_value) = field_val.get(safe_prop).and_then(|v| v.as_str()) {
            let processed = if safe {
                raw_value.to_string()
            } else {
                html_escape::encode_text(raw_value).to_string()
            };
            processed_field[safe_prop] = Value::String(processed);
        }

        ctx.insert("field", &processed_field);
        ctx.insert("globals", &globals);

        let html = tera_snapshot.render(&template_name, &ctx).map_err(|e| {
            tera::Error::msg(format!(
                "render_field: rendering error '{}': {}",
                template_name, e
            ))
        })?;

        Ok(tera::to_value(html).unwrap())
    }
}
