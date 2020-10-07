// do not use this line in your application
use actix_web_dep as actix_web;

use actix_web::{get, App, HttpServer, Result as AwResult};
use maud::{html, Markup};
use std::io;

#[get("/")]
async fn index() -> AwResult<Markup> {
    Ok(html! {
        html {
            body {
                h1 { "Hello World!" }
            }
        }
    })
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
