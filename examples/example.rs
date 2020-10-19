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
        let name = req.param("name")?;
        tera.render_response("hello.html", &context! { "name" => name })
    });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
