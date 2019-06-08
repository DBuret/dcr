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

const dcr_version: &str = "0.1";

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

    // server
    HttpServer::new(|| {
        App::new()
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .service(web::resource("/{dcr_basepath}/health").route(web::get().to(health_handler)))
            .service(
                web::resource("/{dcr_basepath}/health").route(web::put().to(health_toggle_handler)),
            )
            .service(
                web::resource("/{dcr_basepath}/health")
                    .route(web::post().to(health_toggle_handler)),
            )
            .service(web::resource("/{dcr_basepath}/version").route(web::get().to(version_handler)))
            .service(web::resource("/{dcr_basepath}/logger").route(web::put().to(logger_handler)))
            .service(web::resource("/{dcr_basepath}/logger").route(web::post().to(logger_handler)))
            .service(web::resource("/{dcr_basepath}/").route(web::get().to(main_handler)))
            .service(web::resource("/{dcr_basepath}").route(web::get().to(main_handler)))
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
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("{:?}", req))
}

fn health_handler(req: HttpRequest) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("I'm alive")
}

fn health_toggle_handler(req: HttpRequest) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("health toggle handler")
}


fn version_handler(req: HttpRequest, dcr_stamp: String) -> HttpResponse {
    // response
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(dcr_version)
}

fn logger_handler(req: HttpRequest) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("logger handler")
}


/*
fn welcome(req: HttpRequest) -> Result<HttpResponse> {
    println!("{:?}", req);

     // response
    /Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/template.html")))
        */


/// 404 handler
fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

