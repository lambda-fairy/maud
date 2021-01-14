# Web framework integration

Maud includes support for these web frameworks: [Actix], [Iron], [Rocket], and [Rouille].

[Actix]: https://actix.rs/
[Iron]: http://ironframework.io
[Rocket]: https://rocket.rs/
[Rouille]: https://github.com/tomaka/rouille

# Actix

Actix support is available with the "actix-web" feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["actix-web"] }
# ...
```

Actix request handlers can use a `Markup` that implements the `actix_web::Responder` trait.

```rust,no_run
use maud::{html, Markup};
use actix_web::{web, App, HttpServer};

fn index(params: web::Path<(String, u32)>) -> Markup {
    html! {
        h1 { "Hello " (params.0) " with id " (params.1) "!" }
    }
}

fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/user/{name}/{id}", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
}
```

# Iron

Iron support is available with the "iron" feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["iron"] }
# ...
```

With this feature enabled, you can then build a `Response` from a `Markup` object directly. Here's an example application using Iron and Maud:

```rust,no_run
use iron::prelude::*;
use iron::status;
use maud::html;

fn main() {
    Iron::new(|r: &mut Request| {
        let markup = html! {
            h1 { "Hello, world!" }
            p {
                "You are viewing the page at " (r.url)
            }
        };
        Ok(Response::with((status::Ok, markup)))
    }).http("localhost:3000").unwrap();
}
```

`Markup` will set the content type of the response automatically, so you don't need to add it yourself.

# Rocket

Rocket works in a similar way, except using the `rocket` feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["rocket"] }
# ...
```

This adds a `Responder` implementation for the `Markup` type, so you can return the result directly:

```rust,no_run
use maud::{html, Markup};
use rocket::{get, routes};
use std::borrow::Cow;

#[get("/<name>")]
fn hello<'a>(name: Cow<'a, str>) -> Markup {
    html! {
        h1 { "Hello, " (name) "!" }
        p { "Nice to meet you!" }
    }
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch();
}
```

# Rouille

Unlike with the other frameworks, Rouille doesn't need any extra features at all! Calling `Response::html` on the rendered `Markup` will Just Work®.

```rust,no_run
use maud::html;
use rouille::{Response, router};

fn main() {
    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (GET) (/{name: String}) => {
                html! {
                    h1 { "Hello, " (name) "!" }
                    p { "Nice to meet you!" }
                }
            },
            _ => Response::empty_404()
        )
    });
}
```
