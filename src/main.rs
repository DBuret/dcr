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
use std::{env, io, process};


const DCR_VERSION: &str = "0.2";
static HEALTH: AtomicBool = AtomicBool::new(true);

// payload display not implemented
fn main_handler(hb: web::Data<Arc<Handlebars>>, req: HttpRequest) -> HttpResponse {
    info!(
        "{:#?} {} {} - 200 OK",
        req.version(),
        req.method(),
        req.uri()
    );
    // <textarea cols=150 rows=20 readonly>
    let mut header_content = String::new();
    for (key, value) in req.headers() {
        header_content.push_str(&format!("{}: {:#?}\n", key, value));
    }

    let mut env_content = String::new();
    for (key, value) in env::vars() {
        env_content.push_str(&format!("{}: {}\n", key, value));
    }

    let data = json!({
        "version": format!("{:?}",req.version()),
        "method": format!("{:?}",req.method()),
        "uri" : format!("{:?}",req.uri()),
        "header" : header_content,
        "request" : "The dispay of the body request is not implemented yet",
        "env" : env_content
    });

    let body = hb.render("index", &data).unwrap();

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body)
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

// stamp in alpha stage
fn version_handler(stamp: web::Data<String>, req: HttpRequest) -> HttpResponse {
    info!(
        "{:#?} {} {} - 200 OK",
        req.version(),
        req.method(),
        req.uri()
    );
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("{}{:#?}", DCR_VERSION, stamp))
}

// output is in early alpha stage: simple buffer copy and "debug" format
fn logger_handler(body: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
    body.map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, Error>(body)
        })
        .and_then(|body| {
            //let mut output = String::from(format!("{:?}", body));
            info!("{:?}", body);
            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("text/html; charset=utf-8")
                .body("data ingested"))
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
        println!("Problem parsing environment: {}", err);
        process::exit(1);
    });

    let bind_addr = format!("127.0.0.1:{}", config.port);

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

    let sys = actix_rt::System::new("dcr");

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/templates")
        .unwrap();
    let handlebars_ref = web::Data::new(Arc::new(handlebars));

    // server
    HttpServer::new(move || {
        App::new()
            .register_data(handlebars_ref.clone())
            .register_data(web::Data::new(String::from(config.stamp.clone())))
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
            .service(web::resource(&config.path).route(web::route().to(main_handler)))
            .default_service(
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
    .bind(bind_addr)?
    .start();


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


/*
fn debug_handler(body: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
    debug!("debug endpoint - entering...");
    body.map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            debug!("debug endpoint - fold closure...");
            Ok::<_, Error>(body)
        })
        .and_then(|body| {
            debug!("debug endpoint - and_then");
            info!("{:?}", body);
            //Ok(HttpResponse::Ok().finish())
            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("text/html; charset=utf-8")
                .body("data ingested"))
        })
}
*/