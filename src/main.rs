use actix_web::{middleware::Logger, get, web, App, Error, HttpServer,  HttpResponse, HttpRequest};
use env_logger;
use awc::Client;
use actix_web_static_files::ResourceFiles;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

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

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    let body = "
<!DOCTYPE html>
<html>
  <head>
    <title>BROKKOLI</title>
    <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">
    <link rel=\"shortcut icon\" type=\"image/png\" href=\"/static/favicon.ico\"/>
    <link rel=\"stylesheet\" href=\"/static/pico.min.css\">
  </head>
  <body>
    <nav class=\"container-fluid\">
      <ul>
        <li><a href=\"/\"><strong>BROKKOLI</strong></a></li>
      </ul>
    </nav>
    <main class=\"container\">
      <img src=\"/stream/0\">
    </main>

    <footer class=\"container-fluid\">
    </footer>
  </body>
</html>
    ";

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8")
       .body(body))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| {
        let generated = generate();
        App::new()
            .app_data(web::Data::new(Client::default()))
            .service(ResourceFiles::new("/static", generated))
            .service(proxy)
            .service(index)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8070))?
    .run()
    .await
}
