use actix_web::{middleware::Logger, get, App, HttpServer, Responder};
use actix_files::Files;
use askama::Template;
use env_logger;


#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    name: &'a str,
}

#[get("/")]
async fn index() -> impl Responder {
    IndexTemplate{
        name: "test",
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/static", "./static/"))
            .service(index)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
