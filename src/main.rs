#[macro_use]
extern crate log;

extern crate actix_web;
extern crate env_logger;
use std::{env, io};

use actix_files as fs;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};
use std::sync::atomic::{AtomicBool, Ordering};

const DCR_VERSION: &str = "0.1";

static HEALTH: AtomicBool = AtomicBool::new(true);

fn main() -> io::Result<()> {
    // logger init
    env::set_var("RUST_LOG", "dcr=debug");
    env_logger::init();

    // parse env
    let dcr_basepath = env::var("DCR_BASEPATH").expect("DCR_BASEPATH must be set");
    let dcr_port = env::var("DCR_PORT").expect("DCR_PORT must be set");
    let dcr_stamp = env::var("DCR_STAMP").expect("DCR_STAMP must be set");
    let dcr_healthcheck = env::var("DCR_HEALTH").expect("DCR_HEALTH must be set");
    let dcr_logger = env::var("DCR_LOGGER").expect("DCR_LOGGER must be set");

    let sys = actix_rt::System::new("dcr");

    HEALTH.store(true, Ordering::Relaxed);

    // server
    HttpServer::new(|| {
        App::new()
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .service(web::resource("/{dcr_basepath}/health").route(web::get().to(health_handler)))
            .service(
                web::resource("/{dcr_basepath}/health").route(
                    web::route()
                        .guard(guard::Any(guard::Post()).or(guard::Put()))
                        .to(health_toggle_handler),
                ),
            )
            .service(web::resource("/{dcr_basepath}/version").route(web::get().to(version_handler)))
            .service(web::resource("/{dcr_basepath}/logger").route(web::put().to(logger_handler)))
            .service(web::resource("/{dcr_basepath}/logger").route(web::post().to(logger_handler)))
            .service(web::resource("/{dcr_basepath}").route(web::route().to(main_handler)))
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
    .bind("127.0.0.1:8080")?
    .start();

    println!("Starting http server: 127.0.0.1:8080");
    sys.run()
}

fn main_handler(req: HttpRequest) -> HttpResponse {
    let mut body = String::from("<html><body>");
    body.push_str("<H1>Program</H1>");

    body.push_str(&"<H1>Request</H1>");
    body.push_str("<textarea cols=150 readonly>");
    body.push_str(&format!("Protocol: {:?}\n", req.version()));
    body.push_str(&format!("Method: {:?}\n", req.method()));
    body.push_str(&format!("URI: {:?}", req.uri()));
    body.push_str("</textarea>");

    body.push_str("<H1>Headers</H1>");
    body.push_str("<textarea cols=150 rows=20 readonly>");
    for (key, value) in req.headers() {
        body.push_str(&format!("{}: {:#?}\n", key, value));
    }
    body.push_str("</textarea>");

    body.push_str("<H1>Data</H1>");


    body.push_str("<H1>Env</H1>");
    body.push_str("<hr><textarea cols=150 rows=20 readonly>");
    for (key, value) in env::vars() {
        body.push_str(&format!("{}: {}\n", key, value));
    }
    body.push_str("</textarea>\n");

    body.push_str(&"<H1>Debug: http request</H1>");
    body.push_str(
        "<textarea cols=150 rows=20 
    readonly>",
    );
    body.push_str(&format!("{:#?}", req));
    body.push_str("</textarea>");

    body.push_str("</textarea></body></html>\n");

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body)
}

fn health_handler(req: HttpRequest) -> HttpResponse {

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
    let hc = HEALTH.load(Ordering::Relaxed);
    HEALTH.store(!hc, Ordering::Relaxed);
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("healthcheck toggled to {} state", !hc))
}

fn version_handler(req: HttpRequest, dcr_stamp: String) -> HttpResponse {

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(DCR_VERSION)
}

fn logger_handler(req: HttpRequest) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("logger handler")
}


/// 404 handler
fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

