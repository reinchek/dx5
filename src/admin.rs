use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::content::RawHtml;
use rocket::serde::json::{Json, Value};
use rocket::State;
use serde_json::json;
use std::fs;
use std::path::Path;

use crate::contents::{Content, ContentIndex};
use crate::errors::Dx5Error;
use crate::state::AppState;
use gray_matter::Matter;
use gray_matter::engine::YAML;

pub struct AdminAuth;

//--------------------------------------------------------------
// Auth guard - read Authorization: Bearer <token>
//--------------------------------------------------------------
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAuth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let app_state = req.guard::<&State<AppState>>().await.succeeded();
        let expected = app_state
            .and_then(|s| s.app_config.0.admin.as_ref())
            .map(|a| a.token.as_str())
            .unwrap_or("");

        let provided = req
            .headers()
            .get_one("Authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .unwrap_or("");

        if !expected.is_empty() && provided == expected {
            Outcome::Success(AdminAuth)
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------
#[get("/admin")]
pub fn admin_ui() -> RawHtml<&'static str> {
    RawHtml(include_str!("admin_ui.html"))
}

#[get("/admin/api/fields_definitions")]
pub fn api_fields_definitions(_auth: AdminAuth, state: &State<AppState>) -> Json<Value> {
    let defs = &state.fields_config.fields;

    Json(json!({
        "ok": true,
        "definitions": &defs
    }))
}

// ---------------------------------------------------------------------------
// API — Init (languages + content types metadata)
// ---------------------------------------------------------------------------
#[get("/admin/api/init")]
pub fn api_init(_auth: AdminAuth, state: &State<AppState>) -> Json<Value> {
    let languages: Vec<Value> = state
        .app_config
        .0
        .languages
        .0
        .iter()
        .map(|(k, v)| json!({ "code": k, "label": v }))
        .collect();

    let content_types: Vec<Value> = state
        .content_types
        .types
        .iter()
        .map(|(k, v)| {
            json!({
                "key": k,
                "label": v.menu_label,
                "icon": v.menu_icon,
                "route": v.route,
            })
        })
        .collect();

    Json(json!({
        "ok": true,
        "blog": {
            "title": &state.app_config.0.blog.title,
            "author": &state.app_config.0.blog.author,
        },
        "languages": languages,
        "content_types": content_types,
    }))
}

// ---------------------------------------------------------------------------
// API — Contents list
// ---------------------------------------------------------------------------
#[get("/admin/api/contents/<lang>/<type_name>")]
pub fn api_list_contents(
    lang: String,
    type_name: String,
    _auth: AdminAuth,
    state: &State<AppState>,
) -> Json<Value> {
    let def = match state.content_types.types.get(&type_name) {
        Some(d) => d,
        None => {
            return Json(json!({ "ok": false, "error": "content type not found" }));
        }
    };

    // Scan directory directly (bypass .pages cache for admin)
    let dir = format!("{}/{}", def.dir, lang);
    let ext = &def.extension;

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => {
            return Json(json!({ "ok": false, "error": format!("cannot read '{}': {}", dir, e) }));
        }
    };

    let matter = Matter::<YAML>::new();
    let template_stem = "0x00.template";

    let mut items: Vec<Value> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        if stem.starts_with(template_stem) {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some(ext) {
            continue;
        }

        match ContentIndex::load(&lang, stem, &matter, def) {
            Ok(ci) => items.push(json!({
                "id": ci.id,
                "title": ci.title,
                "created": ci.created,
                "updated": ci.updated,
            })),
            Err(e) => eprintln!("[admin][WARN] Skipping '{}': {}", stem, e),
        }
    }

    items.sort_by(|a, b| {
        let a_id = a["id"].as_str().unwrap_or("").to_string();
        let b_id = b["id"].as_str().unwrap_or("").to_string();
        b_id.cmp(&a_id)
    });

    Json(json!({ "ok": true, "items": items }))
}

// ---------------------------------------------------------------------------
// API — Get single content
// ---------------------------------------------------------------------------
#[get("/admin/api/contents/<lang>/<type_name>/<id>")]
pub fn api_get_content(
    lang: String,
    type_name: String,
    id: String,
    _auth: AdminAuth,
    state: &State<AppState>,
) -> Json<Value> {
    let def = match state.content_types.types.get(&type_name) {
        Some(d) => d,
        None => {
            return Json(json!({ "ok": false, "error": "content type not found" }));
        }
    };

    let matter = Matter::<YAML>::new();
    match Content::load(lang, &id, &matter, def) {
        Ok(content) => Json(json!({ "ok": true, "content": content })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ---------------------------------------------------------------------------
// API — Create content
// ---------------------------------------------------------------------------
#[post("/admin/api/contents/<lang>/<type_name>", data = "<body>")]
pub fn api_create_content(
    lang: String,
    type_name: String,
    _auth: AdminAuth,
    body: Json<Value>,
    state: &State<AppState>,
) -> Json<Value> {
    let id = match body.get("id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => {
            return Json(json!({ "ok": false, "error": "Field 'id' is required" }));
        }
    };

    let def = match state.content_types.types.get(&type_name) {
        Some(d) => d,
        None => {
            return Json(json!({ "ok": false, "error": "content type not found" }));
        }
    };

    let path = format!("{}/{}/{}.{}", def.dir, lang, id, def.extension);
    if Path::new(&path).exists() {
        return Json(json!({ "ok": false, "error": format!("Content '{}' already exists", id) }));
    }

    match write_content_yaml(&body.0, &path) {
        Ok(_) => Json(json!({ "ok": true, "id": id })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ---------------------------------------------------------------------------
// API — Update content
// ---------------------------------------------------------------------------
#[put("/admin/api/contents/<lang>/<type_name>/<id>", data = "<body>")]
pub fn api_update_content(
    lang: String,
    type_name: String,
    id: String,
    _auth: AdminAuth,
    body: Json<Value>,
    state: &State<AppState>,
) -> Json<Value> {
    let def = match state.content_types.types.get(&type_name) {
        Some(d) => d,
        None => {
            return Json(json!({ "ok": false, "error": "content type not found" }));
        }
    };

    let path = format!("{}/{}/{}.{}", def.dir, lang, id, def.extension);
    if !Path::new(&path).exists() {
        return Json(json!({ "ok": false, "error": format!("Content '{}' not found", id) }));
    }

    match write_content_yaml(&body.0, &path) {
        Ok(_) => Json(json!({ "ok": true, "id": id })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ---------------------------------------------------------------------------
// API — Delete content
// ---------------------------------------------------------------------------
#[delete("/admin/api/contents/<lang>/<type_name>/<id>")]
pub fn api_delete_content(
    lang: String,
    type_name: String,
    id: String,
    _auth: AdminAuth,
    state: &State<AppState>,
) -> Json<Value> {
    let def = match state.content_types.types.get(&type_name) {
        Some(d) => d,
        None => {
            return Json(json!({ "ok": false, "error": "content type not found" }));
        }
    };

    let path = format!("{}/{}/{}.{}", def.dir, lang, id, def.extension);
    match fs::remove_file(&path) {
        Ok(_) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": format!("Cannot delete '{}': {}", id, e) })),
    }
}

// ---------------------------------------------------------------------------
// API — Blog config
// ---------------------------------------------------------------------------
#[get("/admin/api/config")]
pub fn api_config(_auth: AdminAuth, state: &State<AppState>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "blog": {
            "title": &state.app_config.0.blog.title,
            "author": &state.app_config.0.blog.author,
        }
    }))
}

// ---------------------------------------------------------------------------
// Helper — Serialize JSON to YAML front matter and write to filesystem
// ---------------------------------------------------------------------------
fn write_content_yaml(data: &Value, path: &str) -> Result<(), Dx5Error> {
    let mut fm = data.clone();
    if let Some(obj) = fm.as_object_mut() {
        obj.remove("content");
    }

    let yaml_body = serde_yaml::to_string(&fm).map_err(|e| Dx5Error::parse(e.to_string()))?;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(Dx5Error::from)?;
    }

    fs::write(path, format!("---\n{}---\n", yaml_body)).map_err(Dx5Error::from)?;
    Ok(())
}
