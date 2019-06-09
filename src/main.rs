#[macro_use]
extern crate log;
extern crate actix_web;
extern crate env_logger;

use actix_web::error::ErrorBadRequest;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    dev, error, guard, middleware, web, App, Error, FromRequest, HttpRequest, HttpResponse,
    HttpServer, Result,
};
use futures::{Future, Stream};
use rand;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, io};

const DCR_VERSION: &str = "0.1";
static HEALTH: AtomicBool = AtomicBool::new(true);



fn main() -> io::Result<()> {
    // logger init
    env::set_var("RUST_LOG", "dcr=debug");
    env_logger::init();

    // parse env
    //let dcr_basepath = env::var("DCR_BASEPATH").expect("DCR_BASEPATH must be set");

    let dcr_basepath = "/dcr";
    let dcr_port = env::var("DCR_PORT").expect("DCR_PORT must be set");
    let dcr_stamp = env::var("DCR_STAMP").expect("DCR_STAMP must be set");
    let dcr_healthcheck = env::var("DCR_HEALTH").expect("DCR_HEALTH must be set");
    let dcr_logger = env::var("DCR_LOGGER").expect("DCR_LOGGER must be set");

    info!("Config: version {}{} on port {} and path {}. Inital health answer is {} and logger endpoint is {}",DCR_VERSION, dcr_stamp, dcr_port, dcr_basepath, HEALTH.load(Ordering::Relaxed), dcr_logger);

    let sys = actix_rt::System::new("dcr");

    HEALTH.store(true, Ordering::Relaxed);

    let path_health = format!("{}/health", dcr_basepath);
    let path_version = format!("{}/version", dcr_basepath);
    let path_logger = format!("{}/logger", dcr_basepath);

    // server
    HttpServer::new(move || {
        App::new()
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .service(
                web::resource(&path_health)
                    .route(web::get().to(health_handler))
                    .route(web::put().to(health_toggle_handler))
                    .route(web::post().to(health_toggle_handler)),
            )
            .service(
                web::resource(&path_logger)
                    .route(web::put().to(logger_handler))
                    .route(web::post().to(logger_handler)),
            )
            .service(web::resource(&path_version).route(web::get().to(version_handler)))
            .service(web::resource("/debug").route(web::route().to(debug_handler)))
            .service(web::resource(&dcr_basepath).route(web::route().to(main_handler)))
            // default
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    ),
            )
    })
    .bind(format!("127.0.0.1:{}", dcr_port))?
    .start();

    info!("HTTP server successfully started");
    sys.run()
}

