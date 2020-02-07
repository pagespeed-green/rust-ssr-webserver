use std::io::Read;
use std::fs::File;
use std::io::BufReader;

use actix_files::Files;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder, guard};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
mod routes_list;
use crate::routes_list::get_routes;

lazy_static! {
    static ref ROUTES: HashMap<String, String> = {
        let routes = get_routes();
        routes.iter().cloned().collect()
    };
}

async fn index(req: HttpRequest) -> impl Responder {
    let path_req = req.match_info().query("tail").get(1..).unwrap_or_default().trim().clone();
    let path = if path_req.len() == 0 {
        "home_page"
    } else {
        match ROUTES.get(path_req) {
            Some(r) => r,
            None => "index"
        }
    };

    match std::fs::File::open(format!("static/{}.html", path)) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap_or_default();

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .header("Cache-Control", "no-cache, no-store, max-age=0, must-revalidate")
                .header("pragma", "no-cache")
                .header("x-ua-compatible", "IE=edge, Chrome=1")
                .body(contents)
        },
        Err(e) => {
            println!("index.html is not found - {}", e);

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .header("Cache-Control", "no-cache, no-store, max-age=0, must-revalidate")
                .header("pragma", "no-cache")
                .header("x-ua-compatible", "IE=edge, Chrome=1")
                .body("Resource not found")
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();

    // load ssl keys

    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = rsa_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    println!("Running Web Server --> https://0.0.0.0:3001");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(Files::new("/static", "static"))
            .default_service(
                web::resource("")
                    .route(web::get().to(index))
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    ),
            )
    })
    .bind_rustls("0.0.0.0:3001", config)?
    .run()
    .await
}
