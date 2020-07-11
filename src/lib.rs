//! # Tide-Tera integration This crate exposes [an extension
//! trait](TideTeraExt) that adds two methods to [`tera::Tera`]:
//! [`render_response`](TideTeraExt::render_response) and
//! [`render_body`](TideTeraExt::render_body). It also adds a
//! convenience [`context`] macro for creating ad-hoc tera
//! [`Context`](tera::Context)s.

use std::path::PathBuf;
use tera::{Context, Tera};
use tide::{http::Mime, Body, Response, Result};

/// This extension trait adds two methods to [`tera::Tera`]:
/// [`render_response`](TideTeraExt::render_response) and
/// [`render_body`](TideTeraExt::render_body)
pub trait TideTeraExt {
    /// `render_body` returns a fully-rendered [`tide::Body`] with mime
    /// type set based on the template name file extension using the
    /// logic at [`tide::http::Mime::from_extension`]. This will
    /// return an `Err` variant if the render was unsuccessful.
    ///
    /// ```rust
    /// use tide_tera::prelude::*;
    /// let tera = tera::Tera::new("tests/templates/**/*").unwrap();
    /// let response = tera
    ///     .render_response("good_template.html", &context! { "name" => "tide" })
    ///     .unwrap();
    /// assert_eq!(response.content_type(), Some(tide::http::mime::HTML));
    ///```
    fn render_response(&self, template_name: &str, context: &Context) -> Result;
    /// `render_response` returns a tide Response with a body rendered
    /// with [`render_body`](TideTeraExt::render_body). This will
    /// return an `Err` variant if the render was unsuccessful.
    ///
    /// ```rust
    /// use tide_tera::prelude::*;
    /// let tera = tera::Tera::new("tests/templates/**/*").unwrap();
    /// let body = tera
    ///     .render_body("good_template.html", &context! { "name" => "tide" })
    ///     .unwrap();
    /// assert_eq!(body.mime(), &tide::http::mime::HTML);
    ///```
    fn render_body(&self, template_name: &str, context: &Context) -> Result<Body>;
}

impl TideTeraExt for Tera {
    fn render_body(&self, template_name: &str, context: &Context) -> Result<Body> {
        let string = self.render(template_name, context)?;
        let mut body = Body::from_string(string);

        let path = PathBuf::from(template_name);
        if let Some(extension) = path.extension() {
            if let Some(mime) = Mime::from_extension(extension.to_string_lossy()) {
                body.set_mime(mime)
            }
        }

        Ok(body)
    }

    fn render_response(&self, template_name: &str, context: &tera::Context) -> Result {
        let mut response = Response::new(200);
        response.set_body(self.render_body(template_name, context)?);
        Ok(response)
    }
}

/// this macro simplifies creation of ad-hoc [`tera::Context`]s.
/// ```rust
/// # use tide_tera::context;
/// let mut context = tera::Context::new();
/// context.insert("template-engine", "tera");
/// assert_eq!(context, context! { "template-engine" => "tera" });
/// ```
#[macro_export]
macro_rules! context {
    ($($key:expr => $value:expr,)+) => { context!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let mut _context = ::tera::Context::new();
            $(
                _context.insert($key, &$value);
            )*
            _context
        }
     };
}

pub mod prelude {
    //! exposes [`context`] and [`TideTeraExt].
    pub use super::{context, TideTeraExt};
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::prelude::*;

    #[test]
    fn context() {
        let context = context! {
            "key" => "value"
        };

        assert_eq!(context.into_json()["key"], "value");

        let context = context! { "key1" => "value1", "key2" => "value2" };
        assert_eq!(context.into_json()["key2"], "value2");
    }

    #[async_std::test]
    async fn test_body() {
        let tera = Tera::new("tests/templates/**/*").unwrap();
        let mut body = tera
            .render_body("good_template.html", &context! { "name" => "tide" })
            .unwrap();

        assert_eq!(body.mime(), &tide::http::mime::HTML);

        let mut body_string = String::new();
        body.read_to_string(&mut body_string).await.unwrap();
        assert_eq!(body_string, "hello tide!\n");
    }

    #[async_std::test]
    async fn response() {
        let tera = Tera::new("tests/templates/**/*").unwrap();
        let mut response = tera
            .render_response("good_template.html", &context! { "name" => "tide" })
            .unwrap();

        assert_eq!(response.content_type(), Some(tide::http::mime::HTML));

        let http_response: &mut tide::http::Response = response.as_mut();
        let body_string = http_response.body_string().await.unwrap();
        assert_eq!(body_string, "hello tide!\n");
    }

    #[test]
    fn unknown_content_type() {
        let tera = Tera::new("tests/templates/**/*").unwrap();
        let body = tera
            .render_body("unknown_extension.tide", &context! { "name" => "tide" })
            .unwrap();

        assert_eq!(body.mime(), &tide::http::mime::PLAIN);
    }

    #[test]
    fn no_extension() {
        let tera = Tera::new("tests/templates/**/*").unwrap();
        let body = tera
            .render_body("no_extension", &context! { "name" => "tide" })
            .unwrap();

        assert_eq!(body.mime(), &tide::http::mime::PLAIN);
    }

    #[test]
    fn bad_template() {
        let tera = Tera::new("tests/templates/**/*").unwrap();
        let result = tera.render_body("good_template.html", &context! { "framework" => "tide" });

        assert!(result.is_err());
    }
}
