use rocket::request::FromParam;

pub struct Lang(pub String);

#[rocket::async_trait]
impl<'r> FromParam<'r> for Lang {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        const RESERVED: &[&str] = &["assets", "static", "admin", "api", "css", "js", "fonts", "images", "favicon.ico"];

        if RESERVED.contains(&param) {
            return Err(param);
        }

        // Only accepts valid lang codes: 2-5 alpha-chars.
        if param.len() >= 2 && param.len() <= 5 && param.chars().all(|c| c.is_alphabetic()) {
            Ok(Lang(param.to_string()))
        } else {
            Err(param)
        }
    }
}

pub struct ContentSegment(pub String);

#[rocket::async_trait]
impl<'r> FromParam<'r> for ContentSegment {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        // Refuse any arguments containing a dot (file with an extension)
        if param.contains('.') {
            return Err(param);
        }
        Ok(ContentSegment(param.to_string()))
    }
}
