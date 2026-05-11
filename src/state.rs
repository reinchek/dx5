use rocket::serde::Serialize;
use crate::config::Config;
use crate::contents::ContentTypesConfig;
use crate::fields::FieldsConfig;

pub struct AppState {
    pub app_config: AppConfig,               // App config (dx5.toml)
    pub fields_config: FieldsConfig,         // dx5.fields.toml
    pub content_types: ContentTypesConfig,   // dx5.content_types.toml
    pub audio: AudioSettings,                // playlist JSON
    pub globals: GlobalsState,               // timestamp, ecc.
}

pub struct AppConfig(pub Config);

pub struct GlobalsState {
    pub globals: Globals,
}

#[derive(Serialize, Clone)]
pub struct Globals {
    pub now: String,
}

pub struct AudioSettings {
    pub playlist_json: String,
}
