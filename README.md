# dx5 — file-based CMS for developer blogs

Stack: **Rust + Rocket + Tera + gray_matter**. Content is stored as YAML files. The blog is configured through TOML files in the `config/` directory.

Ready for a **lite headless** approach — all content types (and home pages) can be consumed via the JSON API, making dx5 usable as a backend for decoupled projects, SPAs, or external clients.

![hippo](https://s13.gifyu.com/images/b702k.gif)

---

## Setup

### Scaffolding (recommended)

The dx5 CMS is served with its own `cargo-generate.toml` template file ([cargo-generate](https://cargo-generate.github.io/cargo-generate/)) to bootstrap your blog with an interactive prompt — it will ask for blog title, author, language, admin credentials, and more:

```bash
cargo generate --git https://github.com/reinchek/dx5.git --name my-blog
cd my-blog
# Now run it with cargo run or using docker compose up -d 
```

### Docker Compose

```bash
git clone https://github.com/reinchek/dx5 myblog
cd myblog
docker compose up -d
```

> **Note:** When cloning directly (not via `cargo-generate`), the `Dockerfile` contains `{{ project-name }}` placeholders that must be replaced with your project name first:
> ```bash
> sed -i 's/{{ project-name }}/dx5/g' Dockerfile
> ```

The container runs as a non-root user. On Linux, pass your host UID/GID so mounted volumes have the correct ownership:

```bash
UID=$(id -u) GID=$(id -g) docker compose up
```

On macOS and Windows this is not needed — Docker Desktop handles permission mapping transparently.

### Docker Compose with HTTPS (Caddy)

The project includes a `Caddyfile` and a `caddy` service in `docker-compose.yml` for automatic HTTPS with Let's Encrypt:

1. Replace `your-blog.com` in `Caddyfile` with your actual domain
2. Make sure your domain's DNS points to the server's IP
3. Run:

```bash
UID=$(id -u) GID=$(id -g) docker compose up -d
```

Caddy will proxy requests to dx5 internally on port 8000 and handle SSL termination automatically. Certificates are persisted in Docker named volumes (`caddy_data`, `caddy_config`).

Caddy serves `https://localhost` with a self-signed certificate (Caddy's internal CA). Your browser will show a warning — this is expected for local development. For production, uncomment the domain block in `Caddyfile` for automatic Let's Encrypt certificates.

### Manual (Rust)

```bash
git clone https://github.com/reinchek/dx5 myblog
cd myblog
cp config/dx5.toml.dist config/dx5.toml   # fill in your details
cargo run
```

---

## Project structure

```
myblog/
├── config/
│   ├── dx5.toml                  ← main configuration
│   ├── dx5.fields.toml           ← body field definitions and schemas
│   ├── dx5.content_types.toml    ← content type definitions (posts, codes, etc.)
│   └── locales/
│       ├── en.json               ← English translations
│       ├── ...
│       └── others lang .json files
├── contents/
│   ├── home/                     ← Don't change this one.  
│   │   ├── home.en.yaml
│   │   ├── ...
│   │   └── other per lang home.*.yaml files
│   ├── posts/                       ← Starting content type, you can remove it and do your ones.
│   │   ├── 0x00.template.yaml       ← shared reference template
│   │   ├── en/
│   │   │   ├── 0x01.yaml ... 0x0<N>.yaml
│   │   └── other lang contents folder...
│   ├── ...
│   └── other content type folders
|
├── assets/
│   └── soundtracks/              ← .mp3 files renamed 1.mp3, 2.mp3, ...
├── static/
│   ├── favicon.svg
│   ├── css/
│   │   ├── scrollbar.css
│   │   └── glitch.css
│   ├── js/
│   │   ├── spa-router.js         ← client-side SPA navigation
│   │   ├── lang-switch.js        ← language switcher
│   │   ├── current-time.js       ← system clock display
│   │   └── tailwind-config.js    ← Tailwind CSS configuration
│   └── images/
│       ├── logo.svg
│       ├── base/
│       │   └── env1.jpg ... env9.jpg  ← environmental backgrounds
│       └── you can put all your content's images and media assets.
|
├── templates/
│   ├── base.html.tera            ← base layout
│   ├── home.html.tera            ← homepage template
│   ├── types/
│   │   ├── index.html.tera       ← content listing (with pagination)
│   │   └── single.html.tera      ← single content view
│   ├── components/
│   │   ├── partials/
│   │   │   ├── topbar.tera
│   │   │   ├── footer.tera
│   │   │   └── audio_player_fab.tera
│   │   ├── macros/
│   │   │   ├── t.tera
│   │   │   ├── glitch.tera
│   │   │   └── fields.tera
│   │   ├── dynamic-css/
│   │   │   └── base.tera
│   │   └── fields/               ← one .tera file per field type
│   │       ├── text.tera
│   │       ├── hero_title.tera
│   │       └── ...
│   └── errors/                   ← fallback inline error page
├── src/
│   ├── main.rs
│   ├── routes.rs
│   ├── api.rs                    ← headless JSON API
│   ├── admin.rs                  ← admin panel
│   ├── config.rs
│   ├── contents.rs
│   ├── fields.rs
│   ├── state.rs
│   ├── errors.rs
│   ├── guard.rs
│   ├── locales.rs
│   ├── menu_item.rs
│   ├── watcher.rs                ← filesystem watcher for live reload
│   ├── cache.rs                  ← pagination caching
│   └── audio_metadata.rs
├── Rocket.toml
├── Dockerfile
├── docker-compose.yml
├── cargo-generate.toml           ← project scaffolding template (cargo-generate)
├── Cargo.toml.liquid             ← templated Cargo.toml for scaffolding
```

---

## Configuration

### dx5.toml — main configuration

```toml
[blog]
title        = "My dev blog"
author       = "Your Name"
base_url     = "https://my-blog.dev"
language     = "en"
spa_enabled  = true              # enable client-side SPA routing
# debug_enabled = true           # WIP — exposes debug_data in templates

[languages]
en = "English"
it = "Italiano"

[home]
label = "Home"
icon  = "fa-solid fa-house"      # optional FontAwesome class

[audio]
enabled         = true
soundtracks_dir = "./assets/soundtracks"

[theme]
name                 = "default"
templates_dir        = "./templates"

[server]
port    = 8000
address = "127.0.0.1"

[admin]
enabled = false
token   = "change-this-token-value"
```

> The `DX5_CONFIG` environment variable overrides the config file path:
> ```bash
> DX5_CONFIG=/etc/dx5/prod.toml cargo run --release
> ```

### dx5.content_types.toml — content type definitions

Each content type defines how content is loaded, routed, and displayed:

```toml
[types.post]
dir                       = "./contents/posts"
route                     = "/posts"
menu_icon                 = "fa-regular fa-keyboard"
menu_label                = "Posts"
menu_order                = 1
is_default                = true            # served at /<lang>
ordering                  = "ASC"           # "ASC" or "DESC" (default: "DESC")
pagination_items_per_page = 10
enable_navigation         = true            # prev/next on single view

[types.other_type_N]
# ...
```

### dx5.fields.toml — body field definitions

Body fields are defined separately from the main config. Each section registers a field type, the Tera template used to render it, and the expected data schema.

```toml
[fields.<field_name>]
template    = "components/fields/<field_name>"   # optional — default matches field name
description = "Human-readable description."

[fields.<field_name>.schema]
prop_name = "string"   # "string" | "bool" | "int"
```

### locales/*.json — translations

Translation files in `config/locales/` map keys to localized strings. Keys can be **nested** (grouped by namespace) or **flat**:

```json
{
  "nav": {
    "prev": "← prev",
    "next": "next →",
    "index": "⌗ index"
  },
  "page_title": {
    "suffix_home": "Home",
    "suffix_posts": "Posts"
  },
  "text__explore": "explore",
  "post__index__no_posts_found": "no posts found"
}
```

Both styles work interchangeably. Nested keys are accessed with **dot notation** in templates — the nesting is flattened at load time:

```tera
{{ t(key="nav.prev", lang=lang) }}
{{ t(key="page_title.suffix_home", lang=lang) }}
{{ t(key="text__explore", lang=lang) }}
```

Available languages: `en`, `it`, and whichever you like — just add a `{lang}.json` file.

---

## Content model

### Post front matter

```yaml
id: "0x01"
title: "Post title"
lang: "en"
category: "..."               # optional badge
tags: ["rust", "memory"]      # optional
created: "2026-04-12"
updated: "2026-04-12"         # optional
summary: "..."                # optional meta description
body_fields:
  - ...                       # see Body Fields section
```

### Homepage content

Homepage content is stored as `contents/home/home.{lang}.yaml` and uses the same `Content` struct as posts, rendered through `home.html.tera`.

---

## Body fields

### Available field types

| `type`              | Fields                                                 | Notes                                 |
|---------------------|--------------------------------------------------------|---------------------------------------|
| `text`              | `value`, `safe`                                        | `safe: true` enables raw HTML         |
| `text_glitch`       | `value`, `value_glitch`                                | text with CSS glitch animation        |
| `hero_title`        | `value`, `value_glitch`, `env`, `env_id`, `font_small` | title with glitch effect + env bg  |
| `section_title`     | `value`                                                | section heading                       |
| `cite`              | `value`                                                | inline quotation                      |
| `blockquote`        | `value`, `author`, `reference`                         | blockquote with attribution           |
| `blockquote_figure` | `value`, `author`, `reference`, `figure`               | blockquote with author image          |
| `image`             | `value` (path), `alt`, `caption`                       |                                       |
| `video`             | `src`, `caption`, `autoplay`                           | YouTube, Vimeo, or direct .mp4/.webm  |
| `pre`               | `lang`, `path`, `value`                                | code block with syntax highlight      |
| `terminal`          | `value`, `prompt`, `title`                             | shell session display                 |
| `date`              | `value` (`"2026-04-22"` or `"now"`)                    |                                       |
| `link`              | `href`, `value` (label), `alt`                         |                                       |
| `eol`               | —                                                      | line break                            |
| `spotify_embed`     | `track_id`                                             | embedded Spotify player               |
| `callout`           | `value`, `variant`, `title`, `safe`                    | variants: note, warning, tip, danger  |
| `divider`           | `value`                                                | horizontal rule with optional label   |
| `col`               | `value`, `safe`                                        | column block with Tailwind width       |
| `group`             | `value`                                                | wrapper to group fields (open/close)   |
| `spreaker`          | `episode_id`, `theme`, `title`                         | Spreaker embedded iframe               |

### Full post example

```yaml
---
id: "0x02"
title: "Ownership under pressure"
lang: "en"
category: "MEM_ARCH"
created: "2026-04-22"
summary: "How Rust's ownership model behaves under complex scenarios."
body_fields:
  - type: date
    value: now
  - type: hero_title
    value: Ownership under pressure
    value_glitch: Ownership under pressure
    env: true
    env_id: 5
  - type: text
    value: |
      Post body text, can span multiple lines.
  - type: pre
    lang: rust
    value: |
      fn main() {
          let s = String::from("hello");
          println!("{}", s);
      }
  - type: blockquote
    author: Bjarne Stroustrup
    reference: "The C++ Programming Language"
    value: "C makes it easy to shoot yourself in the foot."
  - type: terminal
    prompt: "$ "
    title: "Build output"
    value: |
      cargo build --release
      Finished release profile
  - type: callout
    variant: warning
    title: "Note"
    value: "This feature is experimental."
  - type: video
    src: "https://www.youtube.com/embed/dQw4w9WgXcQ"
    autoplay: false
---
```

---

## Adding a custom field

No Rust code changes required:

1. Add `[fields.<type>]` to `dx5.fields.toml`
2. Create `templates/components/fields/<type>.tera`
3. Restart the server

### Example: Spotify embed

**1. Add to `dx5.fields.toml`:**
```toml
[fields.spotify_embed]
description = "Embedded Spotify player from a track ID."

[fields.spotify_embed.schema]
track_id = "string"
```

**2. Create `templates/components/fields/spotify_embed.tera`:**
```html
<div class="spotify-embed">
  <iframe
    src="https://open.spotify.com/embed/track/{{ field.track_id }}"
    width="100%" height="152"
    frameborder="0"
    allow="autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture"
    loading="lazy">
  </iframe>
</div>
```

**3. Use in a post:**
```yaml
body_fields:
  - type: spotify_embed
    track_id: "4uLU6hMCjMI75M1A2tKUQC"
```

### Using `render_field` in Tera templates

```html
{% for field in item.body_fields %}
  {{ render_field(field=field, type=field.type) | safe }}
{% endfor %}
```

`render_field` resolves the correct template, builds a context with `field`, and returns rendered HTML. The `| safe` filter is required.

---

## Audio

Files in `soundtracks_dir` must be named sequentially: `1.mp3`, `2.mp3`, etc. Title and artist are read from MP3 ID3 tags. If tags are absent, fallback is `Log_Track_N` / `Unknown_Source`.

The audio player is a floating action button with:
- Slide-up panel with track info, controls, and playlist
- Play/pause, previous, next, mute/unmute
- Progress bar with click-to-seek
- Animated equalizer bars
- Auto-advance to next track
- JavaScript API: `window.DX5Player` with methods `toggle()`, `prev()`, `next()`, `load(index)`, `togglePanel()`, `toggleMute()`, `seek(event)`, `sync(playing)`

To disable audio:
```toml
[audio]
enabled = false
```

---

## Routes

### HTML routes

| Method | Path                           | Description                     |
|--------|--------------------------------|---------------------------------|
| GET    | `/`                            | redirects to default language   |
| GET    | `/<lang>`                      | homepage or default content type|
| GET    | `/<lang>/<type_name>?page=N`   | content listing with pagination |
| GET    | `/<lang>/<type_name>/<id>`     | single content view             |

### Headless JSON API

| Method | Path                           | Description                     |
|--------|--------------------------------|---------------------------------|
| GET    | `/api/<lang>/home`             | homepage content                |
| GET    | `/api/<lang>/<type_name>?page=N` | list items (pagination)       |
| GET    | `/api/<lang>/<type_name>/<id>`   | single item with body_fields  |

Public endpoints — no auth required.

**Examples:**
```
GET /api/en/posts?page=1    → { "ok": true, "items": [...], "pagination": {...} }
GET /api/en/posts/0x01      → { "ok": true, "content": { "id": "...", "body_fields": [...], ... } }
```

### Admin routes

| Method | Path                                          | Description              |
|--------|-----------------------------------------------|--------------------------|
| GET    | `/admin`                                      | admin UI                 |
| GET    | `/admin/api/fields_definitions`               | field type definitions    |
| GET    | `/admin/api/init`                             | languages + content types|
| GET    | `/admin/api/config`                           | blog title and author    |
| GET    | `/admin/api/contents/<lang>/<type_name>`      | list items               |
| GET    | `/admin/api/contents/<lang>/<type_name>/<id>` | get single item          |
| POST   | `/admin/api/contents/<lang>/<type_name>`      | create item              |
| PUT    | `/admin/api/contents/<lang>/<type_name>/<id>` | update item              |
| DELETE | `/admin/api/contents/<lang>/<type_name>/<id>` | delete item              |

Admin routes require `Authorization: Bearer <token>` header.

---

## Templates (Tera)

### Template hierarchy

```
base.html.tera
├── home.html.tera
└── types/
    ├── index.html.tera
    └── single.html.tera
```

### Global Tera functions

| Function     | Purpose                        | Arguments                          |
|--------------|--------------------------------|------------------------------------|
| `render_field` | renders a body field         | `field`, `type`                    |
| `t`          | translates a key               | `key`, `lang` (default: "en")      |
| `app`        | returns blog config object     | (none)                             |

`app()` returns: `{ title, base_url, author, lang, spa_enabled, debug_enabled, start_with_framework, framework }`

### Template variables

**Homepage (`home.html.tera`)**
```
{{ home }}             → HomeConfig
{{ item }}             → Content (homepage content)
{{ lang }}             → current language code
{{ languages }}        → HashMap of available languages
{{ menu_items }}       → Vec<MenuItem>
{{ playlist_data }}    → JSON string of track array
{{ fields_config }}    → field type definitions
{{ globals.now }}      → timestamp at server start
{{ debug_data }}       → all context variables (if debug_enabled)
```

**Content index (`types/index.html.tera`)**
```
{{ items }}            → Vec<ContentIndex>
{{ type_name }}        → content type name ("posts", "codes")
{{ pagination }}       → pagination object
{{ lang }}, {{ languages }}, {{ menu_items }}, {{ fields_config }}, {{ playlist_data }}
```

**Single content (`types/single.html.tera`)**
```
{{ item }}             → Content (full content)
{{ type_name }}        → content type name
{{ enable_navigation }}→ bool
{{ prev_item }}        → [url, title] of previous item
{{ next_item }}        → [url, title] of next item
{{ lang }}, {{ languages }}, {{ menu_items }}, {{ fields_config }}, {{ playlist_data }}, {{ globals.now }}
```

### Pagination object

```json
{
  "current": 1,
  "per_page": 10,
  "pages": [1, 2, 3],
  "total_pages": 3,
  "total_items": 25,
  "has_prev": false,
  "has_next": true
}
```

### Template blocks

- `{% block title %}` — page title
- `{% block description %}` — meta description
- `{% block head %}` — additional head content
- `{% block content %}` — main page content
- `{% block scripts %}` — additional scripts

---

## Internationalization

Languages are defined in `dx5.toml` under `[languages]`. Translation files live in `config/locales/{lang}.json`. Content files are organized by language subdirectories:

```
contents/posts/en/0x01.yaml
contents/posts/it/0x01.yaml
contents/home/home.en.yaml
contents/home/home.it.yaml
```

Use the `t()` function in templates:
```tera
{{ t(key="view_posts", lang=lang) }}
```

---

## SPA mode

Enable client-side navigation in `dx5.toml`:

```toml
[blog]
spa_enabled = true
```

Links with `data-spa` attribute are intercepted. The router fetches HTML via fetch and swaps the `#page-content` div.

---

## Filesystem watcher

Content directories are monitored for changes. When a `.yaml` file is modified, created, or deleted, the pagination cache (`.pages/` folders) is rebuilt automatically. No server restart needed for content changes.

---

## Admin panel

Full CRUD for any content type with multi-language support. Built with **Tailwind CSS + Alpine.js**. Enable in `dx5.toml`:

```toml
[admin]
enabled = true
token   = "your-secret-token"
```

### Features

- **Token authentication** — modal login with session storage
- **Content type selector** — switch between posts, codes, or any custom type
- **Language selector** — manage content per language
- **Item list** — sidebar with all items, sorted by ID
- **Full editor** — front matter fields + body fields builder with add/remove/reorder
- **Dynamic field forms** — each field type shows its specific input fields

Authentication via `Authorization: Bearer <token>` header.

---

## Environment variables

| Variable           | Default           | Description                                      |
|--------------------|-------------------|--------------------------------------------------|
| `DX5_CONFIG`       | `config/dx5.toml` | alternative path for the config file             |
| `DX5_ADMIN_TOKEN`  | (unset)           | override admin token at runtime (overrides config)|
| `RUST_BACKTRACE`   | (unset)           | set to `1` or `full` for backtraces              |
| `RUST_LOG`         | (unset)           | tracing log level (e.g., `info`)                 |

---

## Error handling

| Error                                                         | Cause                                                          |
|---------------------------------------------------------------|----------------------------------------------------------------|
| Content not found                                             | requested content ID does not exist                            |
| Parse error                                                   | malformed YAML front matter                                    |
| I/O error                                                     | directory or file not readable                                 |
| `render_field: type 'X' not defined`                          | field type has no section in `dx5.fields.toml`                 |
| `[WARN] Template not found: components/fields/X.tera`         | field's `.tera` file does not exist                            |

Malformed content files are skipped with a warning — the server does not crash.

For full backtraces: `RUST_BACKTRACE=1 cargo run`.

---

## Extending dx5

### Adding fields

See the "Adding a custom field" section above.

### Adding Rust-side logic

The extension point is `src/fields.rs` — add preprocessing there and inject extra data into the Tera context before rendering.

### Adding content types

Add a new section to `dx5.content_types.toml` with the required properties (`dir`, `extension`, `route`, `menu_label`, `menu_order`).


