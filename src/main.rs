use actix_web::{rt, error, middleware::Logger, get, web, App, Error, HttpServer,  HttpResponse, HttpRequest};
use std::sync::RwLock;
use tinytemplate::TinyTemplate;
use env_logger;
use awc::Client;
use clap::Parser;
use actix_web_static_files::ResourceFiles;
use linux_embedded_hal::{Delay, I2cdev};
use bme280::i2c::BME280;
use actix::{Addr, Actor};
use actix_web_actors::ws;
use serde::{Serialize, Deserialize};

mod sensor;
mod session;
mod camera;

use sensor::BME280Measurement;
use camera::Camera;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[get("/stream/0")]
async fn proxy(
    req: HttpRequest,
    payload: web::Payload,
    client: web::Data<Client>,
    cam: web::Data<RwLock<Camera>>,
    ) -> Result<HttpResponse, Error> {

    
    let stream_url = format!("{}/stream", cam.read().unwrap().base_url.as_str());
    let forwarded_req = client
      .request_from(stream_url, req.head())
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
async fn index(
    tmpl: web::Data<TinyTemplate<'_>>,
    ) -> Result<HttpResponse, Error> {
    let s = tmpl.render("index.html", &serde_json::Value::Null)
        .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(s))
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

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    cam_url: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            cam_url: "http://127.0.0.1:8080".into()
        }
    }
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

    let cfg: Config = confy::load("brokkoli", "config").unwrap();

    let cam = web::Data::new(RwLock::new(Camera{base_url:cfg.cam_url}));
    let cam_clone = cam.clone();
    rt::spawn(async move {
        loop {
            cam_clone.write().unwrap().save_snapshot().await.unwrap();
            rt::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
        }
    });

    let sensorhub = web::Data::new(sensor::SensorHub::new().start());
    let sensorhub_clone = web::Data::clone(&sensorhub);

    rt::spawn(async move {
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
                rt::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        } else {
            println!("I2C device not available");
        }

    });

    HttpServer::new(move || {
        let generated = generate();
        let mut tt = TinyTemplate::new();
        tt.add_template("index.html", INDEX).unwrap();
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(tt))
            .app_data(sensorhub.clone())
            .app_data(cam.clone())
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
static INDEX: &str = include_str!("../templates/index.html");
