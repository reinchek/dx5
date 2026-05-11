use std::fmt;
use std::backtrace::{Backtrace, BacktraceStatus};
use rocket::http::{ContentType, Status};
use rocket::{Request, Response};
use rocket::response::Responder;
use rocket_dyn_templates::tera::escape_html;

#[derive(Debug)]
pub enum Dx5ErrorKind {
    ContentNotFound(String),
    ParseError(String),
    IoError(String),
}
#[derive(Debug)]
pub struct Dx5Error {
    pub kind: Dx5ErrorKind,
    pub context: Vec<String>,
    pub backtrace: Backtrace,
}

impl fmt::Display for Dx5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Dx5ErrorKind::ContentNotFound(id) => write!(f, "Content not found: '{}'", id),
            Dx5ErrorKind::ParseError(msg)     => write!(f, "Parsing error: {}", msg),
            Dx5ErrorKind::IoError(msg)        => write!(f, "I/O error: {}", msg),

        }
    }
}

impl Dx5Error {
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::new(Dx5ErrorKind::ContentNotFound(id.into()))
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        Self::new(Dx5ErrorKind::ParseError(msg.into()))
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::new(Dx5ErrorKind::IoError(msg.into()))
    }

    pub fn new(kind: Dx5ErrorKind) -> Self {
        Self { kind, context: vec![], backtrace: Backtrace::capture() }
    }

    pub fn context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }

    pub fn http_status(&self) -> Status {
        match &self.kind {
            Dx5ErrorKind::ContentNotFound(_) => Status::NotFound,
            _                                => Status::InternalServerError,
        }
    }

    pub fn kind_label(&self) -> &str {
        match self.kind {
            Dx5ErrorKind::ContentNotFound(_) => "ContentNotFound",
            Dx5ErrorKind::ParseError(_) => "ParsingError",
            Dx5ErrorKind::IoError(_) => "IoError",
        }
    }

    pub fn backtrace_lines(&self) -> Vec<String> {
        match self.backtrace.status() {
            BacktraceStatus::Captured   => self
                .backtrace
                .to_string()
                .lines()
                .map(|l| l.to_string())
                .collect(),
            _ => vec![
                "Backtrace is not enabled".to_string(),
                "-> Set RUST_BACKTRACE=1 before start dx5".to_string()
            ],
        }
    }

    fn render_html(&self) -> String {
        let status     = self.http_status();
        let status_str = format!("{} {}", status.code, status.reason().unwrap_or("Error"));
        let message    = escape_html(&self.to_string());

        let context_html = if self.context.is_empty() {
            String::new()
        } else {
            let rows = self.context
                .iter()
                .enumerate()
                .map(|(i, c)| format!(
                    r#"<div class="ctx-item><spa class="n">#{}. </span>{}</div>"#,
                    i + 1,
                    escape_html(c)
                ))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                r#"<p class="section-label">CONTEXT CHAIN</p><div class="ctx-list">{rows}</div>"#
            )
        };
        let bt_html = self
            .backtrace_lines()
            .iter()
            .map(|l| escape_html(l))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            include_str!("utils/error.html"),
            status_str = status_str,
            kind = self.kind_label(),
            status_code = status.code,
            message = message,
            context_html = context_html,
            bt_html = bt_html
        )

    }
}

// Rocket Responder - renders an HTML error page.
impl<'r> Responder<'r, 'static> for Dx5Error {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let status = self.http_status();
        let html   = self.render_html();

        Response::build()
            .status(status)
            .header(ContentType::HTML)
            .sized_body(html.len(), std::io::Cursor::new(html))
            .ok()
    }
}

impl std::error::Error for Dx5Error {}

impl From<std::io::Error> for Dx5Error {
    fn from(e: std::io::Error) -> Self {
        Dx5Error::io(e.to_string())
    }
}
