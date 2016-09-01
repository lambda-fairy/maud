/// Use Maud templates with the Iron web framework.
///
/// # Example
///
/// ```rust,ignore
/// #![feature(plugin)]
/// #![plugin(maud_macros)]
///
/// extern crate iron;
/// extern crate iron_maud;
/// extern crate maud;
///
/// use iron::prelude::*;
/// use iron::status;
/// use iron_maud::Maud;
///
/// fn main() {
///     Iron::new(|r: &mut Request| {
///         let url = r.url.to_string();
///         let body = template! {
///             h1 "Hello, world!"
///             p {
///                 "You are viewing the page at " (url)
///             }
///         };
///         Ok(Response::with((status::Ok, Maud::new(body))))
///     }).http("localhost:3000").unwrap();
/// }
/// ```

use std::io;

extern crate iron;
extern crate maud;

use iron::headers::ContentType;
use iron::modifier::{Modifier, Set};
use iron::modifiers::Header;
use iron::response::{Response, ResponseBody, WriteBody};
use maud::{Template, Utf8Writer};

/// Wraps a Maud template for use in an Iron response.
pub struct Maud<T> { template: Option<T> }

impl<T> Maud<T> {
    pub fn new(template: T) -> Maud<T> where
        T: Template + Send + 'static
    {
        Maud { template: Some(template) }
    }
}

impl<T: Template + Send + 'static> Modifier<Response> for Maud<T> {
    fn modify(self, response: &mut Response) {
        response
            .set_mut(Header(ContentType::html()))
            .set_mut(Box::new(self) as Box<WriteBody>);
    }
}

impl<T: Template + Send> WriteBody for Maud<T> {
    fn write_body(&mut self, body: &mut ResponseBody) -> io::Result<()> {
        if let Some(template) = self.template.take() {
            let mut writer = Utf8Writer::new(body);
            let _ = template(&mut writer);
            writer.into_result()
        } else {
            Ok(())
        }
    }
}

#[test]
fn smoke() {
    let template = |w: &mut std::fmt::Write| w.write_str("Hello, world!");
    let _ = Response::with((iron::status::Ok, Maud::new(template)));
}
