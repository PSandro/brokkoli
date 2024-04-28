use actix_web::{middleware::Logger, get, web, App, Error, HttpServer, Responder, HttpResponse, HttpRequest};
use actix_files::Files;
use askama::Template;
use env_logger;
use awc::Client;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[get("/")]
async fn index() -> impl Responder {
    IndexTemplate{}
}

#[get("/stream/0")]
async fn proxy(
    req: HttpRequest,
    payload: web::Payload,
    client: web::Data<Client>,
    ) -> Result<HttpResponse, Error> {

  let forwarded_req = client
      .request_from("http://127.0.0.1:8080/stream", req.head())
      .no_decompress();

  let res = forwarded_req
      .send_stream(payload)
      .await.unwrap();

  let mut client_resp = HttpResponse::build(res.status());
  for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
      client_resp.insert_header((header_name.clone(), header_value.clone()));
  }
  Ok(client_resp.streaming(res))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(Client::default()))
            .service(Files::new("/static", "./static/"))
            .service(index)
            .service(proxy)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
