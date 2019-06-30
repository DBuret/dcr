#![warn(clippy::all, clippy::nursery)]

#[macro_use]
extern crate log;

/* #[macro_use]
extern crate actix_web; */

/*#[macro_use]
extern crate env_logger;*/

#[macro_use]
extern crate serde_json;

use actix_web::http::StatusCode;
use actix_web::{
    guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};

use futures::{Future, Stream};
use handlebars::Handlebars;
//use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{env, io, process, str};


const DCR_VERSION: &str = "0.2.3";
static HEALTH: AtomicBool = AtomicBool::new(true);

fn main_handler(
    body: web::Payload,
    hb: web::Data<Arc<Handlebars>>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    info!(
        "{:#?} {} {} - 200 OK",
        req.version(),
        req.method(),
        req.uri()
    );

    let mut header_content = String::new();
    for (key, value) in req.headers() {
        header_content.push_str(&format!("{}: {:#?}\n", key, value));
    }

    let mut env_content = String::new();
    for (key, value) in env::vars() {
        env_content.push_str(&format!("{}: {}\n", key, value));
    }

    body.map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, Error>(body)
        })
        .and_then(move |body| {
            let s = match str::from_utf8(&body) {
                Ok(v) => v,
                Err(_e) => "input not displayed, Invalid UTF-8 sequence",
            };

            let data = json!({
                "version": format!("{:?}",req.version()),
                "method": format!("{:?}",req.method()),
                "uri" : format!("{:?}",req.uri()),
                "header" : header_content,
                "input" : s,
                "env" : env_content
            });

            let page = hb.render("index", &data).unwrap();

            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("text/html; charset=utf-8")
                .body(page))
        })
}


fn health_handler(_req: HttpRequest) -> HttpResponse {

    if HEALTH.load(Ordering::Relaxed) {
        HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body("OK")
    } else {
        HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE)
            .content_type("text/html; charset=utf-8")
            .body("KO")
    }
}


fn health_toggle_handler(req: HttpRequest) -> HttpResponse {

    let hc = !HEALTH.load(Ordering::Relaxed);
    HEALTH.store(hc, Ordering::Relaxed);
    info!(
        "{:#?} {} {} - 200 OK - Healthcheck status toggled to: {} ",
        req.version(),
        req.method(),
        req.uri(),
        hc
    );
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("healthcheck toggled to {} state", hc))
}

//
fn version_handler(stamp: web::Data<String>, req: HttpRequest) -> HttpResponse {
    info!(
        "{:#?} {} {} - 200 OK",
        req.version(),
        req.method(),
        req.uri()
    );

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("{}{}", DCR_VERSION, stamp.get_ref()))
}

/// logger endpoint: write payload to info log.
fn logger_handler(body: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
    body.map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, Error>(body)
        })
        .and_then(|body| {
            let s = match str::from_utf8(&body) {
                Ok(v) => v,
                Err(_e) => "output to log refused, Invalid UTF-8 sequence",
            };
            info!("{}", s);
            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("text/html; charset=utf-8")
                .body("data ingested, check the logs."))
        })
}


fn p404(req: HttpRequest) -> HttpResponse {
    info!(
        "{:#?} {} {} - 404 NOT FOUND",
        req.version(),
        req.method(),
        req.uri()
    );
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body("Oops, you requested an unknown location.\n")
}


fn main() -> io::Result<()> {
    // logger init
    env::set_var("RUST_LOG", "dcr=debug");
    env_logger::init();

    // parse env

    let config = Config::new().unwrap_or_else(|err| {
        info!("Problem parsing environment: {}", err);
        process::exit(1);
    });

    let bind_addr = format!("0.0.0.0:{}", config.port);

    info!(
        "Version {}{} on http://{}{}. Healthcheck is {} and logger endpoint is {}",
        DCR_VERSION,
        config.stamp,
        bind_addr,
        config.path,
        if HEALTH.load(Ordering::Relaxed) {
            "OK"
        } else {
            "KO"
        },
        if config.logger_on {
            "active"
        } else {
            "not active"
        },
    );

    // create actix system
    let sys = actix_rt::System::new("dcr");

    // configure HTML template engine
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/templates")
        .unwrap();
    let handlebars_ref = web::Data::new(Arc::new(handlebars));

    // server
    HttpServer::new(move || {
        App::new()
            .register_data(handlebars_ref.clone())
            .register_data(web::Data::new(config.stamp.clone()))
            .wrap(middleware::Logger::default())
            .service(
                web::resource(&config.path_health)
                    .route(web::get().to(health_handler))
                    .route(web::put().to(health_toggle_handler))
                    .route(web::post().to(health_toggle_handler)),
            )
            .service(
                web::resource(&config.path_logger)
                    .route(web::put().to_async(logger_handler))
                    .route(web::post().to_async(logger_handler)),
            )
            .service(web::resource(&config.path_version).route(web::get().to(version_handler)))
            //.service(web::resource("/debug").route(web::route().to_async(debug_handler)))
            .service(web::resource(&config.path).route(web::route().to_async(main_handler)))
            .default_service(
                web::resource("").route(web::get().to(p404)).route(
                    web::route()
                        .guard(guard::Not(guard::Get()))
                        .to(HttpResponse::MethodNotAllowed),
                ),
            )
    })
    .bind(bind_addr)?
    .start();

/* 
#[derive(Default)]
struct ExtractorConfig {
   config: String,
}
  
impl FromRequest for YourExtractor {
   type Error = Error;
   type Future = Result<Self, Self::Error>;
   type Config = ExtractorConfig;
  
   fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
       let cfg = req.app_data::<ExtractorConfig>();
       println!("config data?: {:?}", cfg.unwrap().role);
       ...
   }
}
  
App::new().service(
   resource("/route_with_config")
       .data(ExtractorConfig {
           config: "test".to_string(),
       })
       .route(post().to(handler_fn)),
)


Resource registration. 1.0 version uses generalized resource registration via .service() method.

  App.new().service(
      web::resource("/welcome")
          .route(web::get().to(welcome))
          .route(web::post().to(post_handler))


let app = App::new().service(
        web::scope("/{project_id}")
            .service(web::resource("/path1").to(|| HttpResponse::Ok()))
            .service(web::resource("/path2").to(|| HttpResponse::Ok()))
            .service(web::resource("/path3").to(|| HttpResponse::MethodNotAllowed()))
    );
*/

    sys.run()
}

#[derive(Clone)]
struct Config {
    healthcheck_on: bool,
    logger_on: bool,
    path: String,
    path_health: String,
    path_version: String,
    path_logger: String,
    port: String,
    stamp: String,
}

impl Config {
    fn new() -> Result<Config, &'static str> {
        let path = match env::var("DCR_BASEPATH") {
            Ok(val) => val,
            Err(_e) => String::from("/dcr"),
        };
        let port = match env::var("DCR_PORT") {
            Ok(val) => val,
            Err(_e) => String::from("28657"),
        };
        let stamp = match env::var("DCR_STAMP") {
            Ok(val) => val,
            Err(_e) => String::from(""),
        };
        let healthcheck_on = match env::var("DCR_HEALTHCHECK") {
            Ok(_val) => false,
            Err(_e) => true,
        };
        let logger_on = match env::var("DCR_LOGGER") {
            Ok(_val) => false,
            Err(_e) => true,
        };

        let path_health = format!("{}/health", path);
        let path_version = format!("{}/version", path);
        let path_logger = format!("{}/logger", path);

        HEALTH.store(healthcheck_on, Ordering::Relaxed);

        Ok(Config {
            healthcheck_on,
            logger_on,
            path,
            path_health,
            path_version,
            path_logger,
            port,
            stamp,
        })
    }
}