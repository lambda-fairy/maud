# Web framework integration

Maud includes support for these web frameworks:
[Actix], [Iron], [Rocket], [Rouille], and [Tide].

[Actix]: https://actix.rs/
[Iron]: http://ironframework.io
[Rocket]: https://rocket.rs/
[Rouille]: https://github.com/tomaka/rouille
[Tide]: https://docs.rs/tide/

# Actix

Actix support is available with the "actix-web" feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["actix-web"] }
# ...
```

Actix request handlers can use a `Markup`
that implements the `actix_web::Responder` trait.

```rust,no_run
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
```

# Iron

Iron support is available with the "iron" feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["iron"] }
# ...
```

With this feature enabled,
you can then build a `Response` from a `Markup` object directly.
Here's an example application using Iron and Maud:

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

`Markup` will set the content type of the response automatically,
so you don't need to add it yourself.

# Rocket

Rocket works in a similar way,
except using the `rocket` feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["rocket"] }
# ...
```

This adds a `Responder` implementation for the `Markup` type,
so you can return the result directly:

```rust,no_run
use maud::{html, Markup};
use rocket::{get, routes};

#[get("/<name>")]
fn hello(name: &str) -> Markup {
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

Unlike with the other frameworks,
Rouille doesn't need any extra features at all!
Calling `Response::html` on the rendered `Markup` will Just Work®.

```rust,no_run
use maud::html;
use rouille::{Response, router};

fn main() {
    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (GET) (/{name: String}) => {
                Response::html(html! {
                    h1 { "Hello, " (name) "!" }
                    p { "Nice to meet you!" }
                })
            },
            _ => Response::empty_404()
        )
    });
}
```

# Tide

Tide support is available with the "tide" feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["tide"] }
# ...
```

This adds an implementation of `From<PreEscaped<String>>` for the `Response` struct.
Once provided, callers may return results of `html!` directly as responses:

```rust,no_run
use maud::html;
use tide::Request;
use tide::prelude::*;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/hello/:name").get(|req: Request<()>| async move {
        let name: String = req.param("name")?.parse()?;
        Ok(html! {
            h1 { "Hello, " (name) "!" }
            p { "Nice to meet you!" }
        })
    });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

# Axum

Axum support is available with the "axum" feature:

```toml
# ...
[dependencies]
maud = { version = "*", features = ["axum"] }
# ...
```

This adds an implementation of `IntoResponse` for `Markup`/`PreEscaped<String>`.
This then allows you to use it directly as a response!

```rust,no_run
use maud::{html, Markup};
use axum::{Router, handler::get};

async fn hello_world() -> Markup {
    html! {
        h1 { "Hello, World!" }
    }
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```
