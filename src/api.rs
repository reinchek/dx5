use crate::contents::Content;
use crate::guard::{ContentSegment, Lang};
use crate::state::AppState;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use rocket::serde::json::{Json, Value};
use rocket::State;
use serde_json::json;

#[get("/<lang>/home", rank = 1)]
pub fn api_home(lang: Lang, state: &State<AppState>) -> Json<Value> {
    let matter = Matter::<YAML>::new();
    let first_type = &state.content_types.types.keys().last().unwrap().clone();
    match Content::load_home(Some(lang.0), &state.app_config, &matter, &state.content_types.types.get(first_type).unwrap()) {
        Ok(content) => Json(json!({
            "ok": true,
            "content": content,
        })),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

#[get("/<lang>/<type_name>?<page>", rank = 2)]
pub fn api_contents(
    lang: Lang,
    type_name: ContentSegment,
    page: Option<usize>,
    state: &State<AppState>,
) -> Json<Value> {
    let matter = Matter::<YAML>::new();
    let def = match state.content_types.types.get(&type_name.0) {
        Some(d) => d,
        None => return Json(json!({ "ok": false, "error": "content type not found" })),
    };

    match Content::all(&lang.0, &matter, def, page) {
        Ok((items, pagination)) => {
            Json(json!({ "ok": true, "items": items, "pagination": pagination }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[get("/<lang>/<type_name>/<id>")]
pub fn api_content(
    lang: Lang,
    type_name: ContentSegment,
    id: &str,
    state: &State<AppState>,
) -> Json<Value> {
    let matter = Matter::<YAML>::new();
    let def = match state.content_types.types.get(&type_name.0) {
        Some(d) => d,
        None => return Json(json!({ "ok": false, "error": "content type not found" })),
    };

    match Content::load(lang.0.clone(), id, &matter, def) {
        Ok(content) => Json(json!({ "ok": true, "content": content })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}
