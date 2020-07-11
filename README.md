# Tide-Tera Integration

This crate exposes an extension trait that adds two functions to
`tera::Tera`: `render_response` and `render_body`. It also adds a
convenience `context` macro for creating ad-hoc tera Contexts.

---

<a href="https://crates.io/crates/tide-tera">
<img src="https://img.shields.io/crates/v/tide-tera.svg?style=flat-square"
alt="Crates.io version" />
</a>

<a href="https://docs.rs/tide-tera">
<img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
alt="docs.rs docs" />
</a>

---

```rust
use tera::Tera;
use tide_tera::prelude::*;

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();

    let mut tera = Tera::new("examples/templates/**/*")?;
    tera.autoescape_on(vec!["html"]);

    let mut app = tide::with_state(tera);

    app.at("/:name").get(|req: tide::Request<Tera>| async move {
        let tera = req.state();
        let name: String = req.param("name")?;
        tera.render_response("hello.html", &context! { "name" => name })
    });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
```
