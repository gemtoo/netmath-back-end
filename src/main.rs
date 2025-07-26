#[macro_use]
extern crate log;
// backend/src/main.rs
mod tracing;
use ::tracing::{Instrument, debug, error, info, warn};
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, http, post, web};
use std::process::Command;

#[derive(serde::Deserialize, Debug)]
struct CalcRequest {
    subnet: String,
}

#[::tracing::instrument]
#[post("/api")]
async fn calculate(req: web::Json<CalcRequest>) -> HttpResponse {
    match is_whitelisted(&req.subnet) {
        true => {
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
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Command failed: {}", e))
                }
            }
        }
        false => {
            HttpResponse::Forbidden().body("This pattern is restricted.")
        }
    }
}

// Add this OPTIONS handler
#[actix_web::options("/api")]
async fn handle_options() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing::init();
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["POST"])
            .allowed_headers(vec![http::header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(calculate)
            .service(handle_options) // Register the OPTIONS handler
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}

const ALLOWED_CHARS: &str = "0123456789abcdefxABCDEFX.:/";
const ALLOWED_LETTERS: &str = "abcdefxABCDEFX";

fn is_whitelisted(input: &str) -> bool {
    if input.len() >= 2 {
        let has_dot = input.contains('.');
        let has_allowed_letter = input.chars().any(|c| ALLOWED_LETTERS.contains(c));
        if has_dot && has_allowed_letter {
            return false;
        }
    }
    if input.len() >= 80 {
        return false;
    }
    input.chars().all(|c| ALLOWED_CHARS.contains(c))
}
