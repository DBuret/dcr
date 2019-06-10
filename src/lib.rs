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

//////  

use futures::{Future, Stream};
use actix_web::{web, error, App, Error, HttpResponse};

/// extract binary data from request
fn index(body: web::Payload) -> impl Future<Item = HttpResponse, Error = Error>
{
    body.map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, Error>(body)
         })
         .and_then(|body| {
             format!("Body {:?}!", body);
             Ok(HttpResponse::Ok().finish())
         })
}

fn main() {
    let app = App::new().service(
        web::resource("/index.html").route(
            web::get().to_async(index))
    );
}
///////



/// debug handler
fn debug_handler(body: web::Payload) -> HttpResponse {
    debug!("entering debug zone");

    // add here the reading of the body stream
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("display of payload not yet implemented")
}


fn main_handler(req: HttpRequest) -> HttpResponse {
    info!(
        "{:#?} {} {} - 200 OK",
        req.version(),
        req.method(),
        req.uri()
    );
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
    body.push_str("</textarea>");

    body.push_str("</body></html>\n");

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

//nts: add stamp here
fn version_handler(req: HttpRequest, dcr_stamp: String) -> HttpResponse {
    info!(
        "{:#?} {} {} - 200 OK",
        req.version(),
        req.method(),
        req.uri()
    );
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("{}", DCR_VERSION))
}

//nts: get input
fn logger_handler(req: HttpRequest) -> HttpResponse {
    info!("");
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("input written to log")
}

/// 404 handler
fn p404(req: HttpRequest) -> HttpResponse {
    info!(
        "{:#?} {} {} - 404 NOT FOUND",
        req.version(),
        req.method(),
        req.uri()
    );
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body("NOT FOUND")
}

