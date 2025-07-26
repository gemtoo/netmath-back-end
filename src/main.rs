#[macro_use]
extern crate log;
// backend/src/main.rs
mod tracing;
use actix_cors::Cors;
use actix_web::{post, web, App, HttpServer, HttpResponse, http};
use std::process::Command;
use ::tracing::{Instrument, debug, error, info, warn};

#[derive(serde::Deserialize, Debug)]
struct CalcRequest {
    subnet: String,
}

#[::tracing::instrument]
#[post("/api")]
async fn calculate(req: web::Json<CalcRequest>) -> HttpResponse {
    let output = Command::new("subnetcalc")
        .arg(&req.subnet)
        .arg("-nocolor")
        .arg("-n")
        .output();

    match output {
        Ok(output) => {
            let result = if !output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stdout).into_owned()
            } else {
                String::from_utf8_lossy(&output.stderr).into_owned()
            };

            let processed = result
                .replace('\n', "<br>")
                .replace("ERROR: ", "")
                .replace('!', ".")
                .replace("{ ", "")
                .replace(" }", "");

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(processed)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Command failed: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing::init();
    HttpServer::new(|| {
        // Create CORS middleware inside the app factory closure
        let cors = Cors::default()
            .allow_any_origin() // Allow all origins (for development)
            .allowed_methods(vec!["POST"])
            .allowed_headers(vec![http::header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            //.wrap(cors)
            .service(calculate)
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
