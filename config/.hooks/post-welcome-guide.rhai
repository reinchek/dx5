// dx5 - post-generation welcome guide
// Runs automatically after `cargo generate` completes.

let project = variable::get("project-name");
let author  = variable::get("blog_author");

print("");
print("╔══════════════════════════════════════════════════════════════╗");
print("║                  dx5 // scaffold complete                    ║");
print("╠══════════════════════════════════════════════════════════════╣");
print(`║  project : ${project}`);
print(`║  author  : ${author}`);
print("╠══════════════════════════════════════════════════════════════╣");
print("║                                                              ║");
print("║  STEP 1 - configure                                          ║");
print("║  Open dx5.toml and fill in your blog title, base_url,        ║");
print("║  author and language. That file is your single source        ║");
print("║  of truth for the whole site.                                ║");
print("║                                                              ║");
print("║  STEP 2 - set the admin token                                ║");
print("║  [admin]                                                     ║");
print("║  enabled = true                                              ║");
print("║  token   = \"your-secret-token\"                               ║");
print("║  ⚠  never commit a real token to version control.            ║");
print("║                                                              ║");
print("║  STEP 3 - write your first post                              ║");
print("║  Copy contents/posts/0x00.template.yaml                      ║");
print("║  → contents/posts/<lang>/0x01.yaml                           ║");
print("║  Edit the front matter and add body_fields entries.          ║");
print("║                                                              ║");
print("║  STEP 4 - run                                                ║");
print("║  $ cargo run                                                 ║");
print("║  Server starts at http://127.0.0.1:8000                      ║");
print("║  Admin UI at  http://127.0.0.1:8000/admin                    ║");
print("║                                                              ║");
print("║  STEP 5 - add a custom field (optional)                      ║");
print("║  1. Add [fields.<type>] to config/dx5.fields.toml            ║");
print("║  2. Create templates/fields/<type>.tera                      ║");
print("║  3. Restart - no Rust code needed.                           ║");
print("║                                                              ║");
print("╠══════════════════════════════════════════════════════════════╣");
print("║  docs  →  README.md                                          ║");
print("║  bugs  →  github.com/reinchek/dx5/issues                     ║");
print("╚══════════════════════════════════════════════════════════════╝");
print("");
