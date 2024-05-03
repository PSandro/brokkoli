use actix_web::{middleware::Logger, get, web, App, Error, HttpServer,  HttpResponse, HttpRequest};
use env_logger;
use awc::Client;
use clap::Parser;
use actix_web_static_files::ResourceFiles;
use linux_embedded_hal::{Delay, I2cdev};
use bme280::i2c::BME280;
use actix::{Addr, Actor};
use actix_web_actors::ws;

mod sensor;
mod session;

use sensor::BME280Measurement;

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
    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8")
       .body(include_str!("index.html")))
}

#[get("/ws")]
async fn websocket_route(
        req: HttpRequest,
        stream: web::Payload,
        srv: web::Data<Addr<sensor::SensorHub>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsChatSession {
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[derive(clap::Parser, Debug)]
struct CliArguments {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value_t = 5000)]
    port: u16,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = CliArguments::parse();
        log::info!(
        "starting HTTP server at http://{}:{}",
        &args.host,
        args.port
    );

    let sensorhub = web::Data::new(sensor::SensorHub::new().start());
    let sensorhub_clone = web::Data::clone(&sensorhub);

    std::thread::spawn(move || {
        if let Ok(i2c_bus) = I2cdev::new("/dev/i2c-1") {
            let mut bme280 = BME280::new_primary(i2c_bus);
            let mut delay = Delay;
            bme280.init(&mut delay).unwrap();
            loop {
                let measurements = bme280.measure(&mut delay).unwrap();
                let msg = BME280Measurement{
                    humidity: measurements.humidity,
                    pressure: measurements.pressure,
                    temperature: measurements.temperature
                };
                sensorhub_clone.do_send(msg);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        } else {
            println!("I2C device not available");
        }

    });

    HttpServer::new(move || {
        let generated = generate();
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(sensorhub.clone())
            .service(ResourceFiles::new("/static", generated))
            .service(proxy)
            .service(index)
            .service(websocket_route)
            .wrap(Logger::default())
    })
    .bind((args.host, args.port))?
    .run()
    .await


}
