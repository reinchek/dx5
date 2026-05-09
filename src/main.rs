#[macro_use]
extern crate rocket;
mod admin;
mod api;
mod audio_metadata;
mod config;
mod contents;
mod errors;
mod fields;
mod guard;
mod menu_item;
mod state;
mod routes;
mod locales;
mod cache;
mod watcher;

use crate::audio_metadata::get_playlist;
use crate::config::Config;
use crate::contents::ContentTypesConfig;
use crate::fields::{make_fields_renderer, FieldsConfig};
use crate::locales::{load_translations, make_translator};
use crate::routes::{generic_content, generic_contents, home, index, root};
use crate::state::{AppConfig, AppState, AudioSettings, Globals, GlobalsState};
use chrono::Local;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use crate::cache::split_contents_in_pages_folders;
use crate::watcher::start_filesystem_watcher;

// ---------------------------------------------------------------------------
// Launch
// ---------------------------------------------------------------------------
#[launch]
fn rocket() -> _ {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("[FATAL] {}", e);
        std::process::exit(1);
    });

    // Load fields definition from dx5.fields.toml.
    let fields_config: FieldsConfig =
        FieldsConfig::load().expect("[FATAL] Field to load fields configuration file.");

    let playlist = if config.audio.enabled {
        get_playlist(Some(&config.audio.soundtracks_dir))
    } else {
        vec![]
    };

    let playlist_json = serde_json::to_string(&playlist).unwrap_or_else(|_| "[]".to_string());
    let globals = Globals {
        now: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    let state = AppState {
        app_config: AppConfig(config.clone()),
        content_types: ContentTypesConfig::init(),
        fields_config: fields_config.clone(),
        audio: AudioSettings { playlist_json },
        globals: GlobalsState {
            globals: globals.clone(),
        },
    };

    if let Err(e) = start_filesystem_watcher(&config) {
        eprintln!("[FS][warn] Filesystem watcher not started: {}", e);
    }
    for (_type_name, type_def) in &state.content_types.types {
        let languages = config.languages.0.keys().collect::<Vec<&String>>();
        split_contents_in_pages_folders(languages, type_def, true).expect("TODO: panic message");
    }

    rocket::build()
        .manage(state)
        // Routes
        .mount(
            "/",
            routes![root, home, index, generic_contents, generic_content],
        )
        // Headless API
        .mount("/api", routes![api::api_contents, api::api_content, api::api_home])

        // Static files mount
        .mount("/assets", FileServer::from("assets"))
        .mount("/static", FileServer::from("static"))
        .mount("/", FileServer::from("static").rank(20)) // ← catch favicon, robots.txt, ecc.
        // Admin: only mounted if the [admin] section exists in dx5.toml file and is `enabled = true`
        .mount(
            "/",
            if config.admin.as_ref().map(|a| a.enabled).unwrap_or(false) {
                routes![
                    admin::admin_ui,
                    admin::api_config,
                    admin::api_fields_definitions,
                    admin::api_init,
                    admin::api_list_contents,
                    admin::api_get_content,
                    admin::api_create_content,
                    admin::api_update_content,
                    admin::api_delete_content,
                ]
            } else {
                routes![]
            },
        )
        // Fairing
        // .attach(Template::fairing())
        .attach(Template::custom(move |engines| {
            let tmpl_dir = &config.theme.templates_dir;

            let translations_map = Arc::new(load_translations());
            let translations_for_fairing = translations_map.clone();

            engines.tera.register_function("t", make_translator(translations_for_fairing));

            // 1. Load all field's templates with &mut engines.tera.
            for (field_type, def) in &fields_config.fields {
                let tname = def
                    .template
                    .clone()
                    .unwrap_or_else(|| format!("components/fields/{}", field_type));
                let path = std::path::PathBuf::from(format!("{}/{}.tera", tmpl_dir, tname));

                if path.exists() {
                    if let Err(e) = engines.tera.add_template_file(path, Some(&tname)) {
                        eprintln!("[WARN] Template '{}' not loaded: {}", tname, e);
                    }
                } else {
                    eprintln!("[WARN] Template not found: {}/{}.tera", tmpl_dir, tname);
                }
            }

            // Register the "render_field" global function.
            engines.tera.register_function(
                "render_field",
                make_fields_renderer(
                    fields_config.clone(),
                    globals.clone(),
                    // Immutable snapshot of tera, with all templates already inside it.
                    engines.tera.clone(),
                ),
            );

            // Register the "app" global function (used to access to all dx5.toml configurations).
            let app = serde_json::json!({
                "title":         config.blog.title,
                "base_url":      config.blog.base_url,
                "author":        config.blog.author,
                "lang":          config.blog.language,
                "spa_enabled":   config.blog.spa_enabled,
                "debug_enabled": config.blog.debug_enabled.unwrap_or(false),
                "start_with_framework": config.theme.start_with_framework,
                "framework": config.theme.framework
            });

            engines.tera.register_function("app", move |_: &HashMap<String, Value>| {
                Ok(app.clone())
            });
        }))
}
