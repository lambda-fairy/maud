# Web framework integration

Maud includes support for these web frameworks:
[Actix], [Iron], [Rocket], [Rouille], and [Warp].

[Actix]: https://actix.rs/
[Iron]: http://ironframework.io
[Rocket]: https://rocket.rs/
[Rouille]: https://github.com/tomaka/rouille
[Warp]: https://github.com/seanmonstar/warp

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
#![feature(decl_macro)]

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

# Warp

Warp also doesn't need any extra features!
Just call `String::from` on the rendered `Markup` .

```rust,no_run
use maud::{DOCTYPE, html, PreEscaped};
use warp::Filter;

#[tokio::main]
async fn main() {
    let hello_routes =
        warp::get()
        .and(
            // GET /hello/warp => 200 OK with body "(...)Hello, warp!(...)"
            warp::path!("hello" / String)
                .map(|name: String| hello_markup(&name))
                .map(|body| warp::reply::html(body))
            // or GET /hello => 200 OK with body "(...)Hello, there!(...)"
            .or(warp::path!("hello")
                .map(||     hello_markup("there"))
                .map(|body| warp::reply::html(body))
            )
        );

    warp::serve(hello_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn hello_markup(name: &str) -> String {
    String::from(html! {
        (DOCTYPE)
        html {
            head {
                style {(PreEscaped(r#"
                    body {
                        padding: 20px 20px;
                        background-color: yellow;
                    }
                "#))}
            }
            body {
                p { "Hello, " (name) "!" }
            }
        }
    })
}
```
