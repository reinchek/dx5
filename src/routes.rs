use crate::contents::{Content, DEFAULT_TYPES_TEMPLATE_FOLDER};
use crate::errors::Dx5Error;
use crate::guard::{ContentSegment, Lang};
use crate::state::AppState;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use tracing::error;

#[get("/")]
pub fn root() -> Redirect {
    Redirect::to(uri!("/en"))
}

#[get("/<lang>", rank = 1)]
pub fn home(lang: Lang, state: &State<AppState>) -> Result<Template, Dx5Error> {
    let matter = Matter::<YAML>::new();
    let menu_items = &state.content_types.as_menu_items();
    let home = &state.app_config.0.home;
    let ct_def = &state.content_types.types.iter().last().unwrap();

    let lang = lang.0.clone();

    let item = Content::load_home(Some(lang.clone()), &state.app_config, &matter, ct_def.1)
        .map_err(|e| e.context("route: home page load"))?;

    Ok(Template::render(
        "home",
        context! {
            home: &home,
            lang: &lang,
            item: &item,
            languages: &state.app_config.0.languages.0,
            globals: &state.globals.globals,
            fields_config: &&state.fields_config.fields,
            menu_items: &menu_items,
            playlist_data: &&state.audio.playlist_json,
            debug_data: context! {
                home: &home,
                lang: &lang,
                item: &item,
                languages: &state.app_config.0.languages.0,
                globals: &state.globals.globals,
                fields_config: &&state.fields_config.fields,
                menu_items: &menu_items,
                playlist_data: &&state.audio.playlist_json,
            }
        },
    ))
}

#[get("/<lang>", rank = 2)]
pub fn index(lang: Lang, state: &State<AppState>) -> Result<Template, Dx5Error> {
    let matter = Matter::<YAML>::new();
    let menu_items = &state.content_types.as_menu_items();
    let (_ct_name, ct_def) = &state.content_types.get_default().ok_or_else(|| {
        Dx5Error::parse("No `is_default` content type found in dx5.content_types.toml")
    })?;

    let template_path = format!(
        "{}/index",
        &ct_def
            .template
            .clone()
            .unwrap_or(DEFAULT_TYPES_TEMPLATE_FOLDER.to_string())
    );

    let lang = lang.0.clone();

    let items = Content::all(&lang, &matter, ct_def, None)
        .map_err(|e| e.context(format!("route: index, lang={}", lang)))?;

    Ok(Template::render(
        template_path,
        context! {
            items: &items.0,
            lang: &lang,
            type_name: _ct_name,
            languages: &state.app_config.0.languages.0,
            menu_items: &menu_items,
            fields_config: &state.fields_config.fields,
            playlist_data: &state.audio.playlist_json,
            debug_data: context! {
                items: &items.0,
                lang: &lang,
                menu_items: &menu_items,
                fields_config: &state.fields_config.fields,
                playlist_data: &state.audio.playlist_json,
            }
        },
    ))
}

#[get("/<lang>/<type_name>?<page>")]
pub fn generic_contents(
    lang: Lang,
    type_name: ContentSegment,
    page: Option<usize>,
    state: &State<AppState>,
) -> Result<Template, Dx5Error> {
    let path = format!("{}/{}", &lang.0, &type_name.0);
    let matter = Matter::<YAML>::new();
    let def = state.content_types.types.get(&type_name.0).ok_or_else(|| {
        error!(
            path,
            "Content type '{}' not registered in dx5.content_types.toml", &type_name.0
        );
        Dx5Error::not_found(&type_name.0)
            .context("route: content type '{}' not registered in dx5.content_types.toml")
    })?;

    let items = Content::all(&lang.0, &matter, def, page).map_err(|e| {
        error!(
            path,
            "Error on generic_contents, lang={}, error={}",
            &lang.0,
            e.to_string()
        );
        e.context(format!(
            "route: generic_contents type={} lang={}",
            &type_name.0, &lang.0
        ))
    })?;

    let menu_items = state.content_types.as_menu_items();

    let template_path = format!(
        "{}/index",
        &def.template
            .clone()
            .unwrap_or(DEFAULT_TYPES_TEMPLATE_FOLDER.to_string())
    );

    Ok(Template::render(
        template_path,
        context! {
            items: items.0,
            lang: lang.0,
            type_def: &state.content_types.types.get(&type_name.0).unwrap() ,
            globals: &&state.globals.globals,
            type_name: type_name.0,
            menu_items,
            languages: &state.app_config.0.languages.0,
            fields_config: &state.fields_config.fields,
            playlist_data: &state.audio.playlist_json,
            pagination: items.1,
        },
    ))
}
#[get("/<lang>/<type_name>/<id>")]
pub fn generic_content(
    lang: Lang,
    type_name: ContentSegment,
    id: &str,
    state: &State<AppState>,
) -> Result<Template, Dx5Error> {
    let path = format!("{}/{}/{}", lang.0.clone(), type_name.0.clone(), id);
    let matter = Matter::<YAML>::new();
    let def = state.content_types.types.get(&type_name.0).ok_or_else(|| {
        error!(
            path,
            "Content type '{}' not registered in dx5.content_types.toml",
            type_name.0.clone()
        );
        Dx5Error::not_found(type_name.0.clone()).context("route: content type '{}' not found")
    })?;

    // Check if post's navigation feature (`enable_navigation = <bool>` in dx5.content_types.toml) is enabled.
    let nav_enabled = state
        .content_types
        .get_enable_navigation(&type_name.0.clone())
        .unwrap_or(false);

    let mut all_items: Vec<String> = Vec::new();
    let mut all_items_titles = Vec::new();
    let mut prev_item = None;
    let mut next_item = None;

    if nav_enabled {
        let items = Content::all(&lang.0, &matter, def, None).map_err(|err| {
            error!(
                path,
                "Error on generic_content, lang={}, error={}",
                lang.0.clone(),
                err.to_string()
            );
            err.context(format!("route: generic_contents type={}", &lang.0))
        })?;

        for content in items.0 {
            let content_id = content.id.clone();
            if !all_items.contains(&content_id) {
                all_items.push(content_id);
                all_items_titles.push(content.title.clone());
            }
        }

        let id_pos = all_items.iter().position(|x| *x == id).unwrap_or(0);
        let prev_pos = id_pos.checked_sub(1).unwrap_or_else(|| all_items.len() - 1);
        let next_pos = if id_pos < all_items.len() - 1 {
            id_pos.checked_add(1).unwrap_or(0)
        } else {
            all_items.len() - 1
        };
        let prev_id = &all_items[prev_pos];
        let next_id = &all_items[next_pos];

        prev_item = Some((prev_id, &all_items_titles[prev_pos]));
        next_item = Some((next_id, &all_items_titles[next_pos]));
    }

    let item = Content::load(lang.0.clone(), id, &matter, def).map_err(|e| {
        error!(path, "{}", e.to_string());
        e.context(format!(
            "route: generic_content type={} id={} lang={}",
            type_name.0, id, &lang.0
        ))
    })?;

    let menu_items = state.content_types.as_menu_items();

    let template_path = format!(
        "{}/single",
        def.template
            .clone()
            .unwrap_or(DEFAULT_TYPES_TEMPLATE_FOLDER.to_string())
    );

    Ok(Template::render(
        template_path,
        context! {
            item,
            type_name: &type_name.0,
            globals: &state.globals.globals,
            enable_navigation: nav_enabled,
            prev_item: if nav_enabled {
                [format!("/{}/{}/{}", lang.0.clone(), &type_name.0, &prev_item.unwrap().0), prev_item.unwrap().1.to_string()]
            } else {
                [String::new(), String::new()]
            },
            next_item: if nav_enabled {
                [format!("/{}/{}/{}", &lang.0, &type_name.0, &next_item.unwrap().0), next_item.unwrap().1.to_string()]
            } else {
                [String::new(), String::new()]
            },
            lang: lang.0,
            languages: &state.app_config.0.languages.0,
            menu_items,
            fields_config: &state.fields_config.fields,
            playlist_data: &state.audio.playlist_json,
        },
    ))
}
