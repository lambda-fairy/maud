// https://github.com/sunng87/handlebars-rust/blob/master/benches/bench.rs

#![feature(test)]

extern crate test;

use handlebars::{to_json, Handlebars};
use serde_json::value::{Map, Value as Json};

static SOURCE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {{#each teams}}
      <li class=\"{{#if @first}}champion{{/if}}\">
      <b>{{name}}</b>: {{score}}
      </li>
    {{/each}}
    </ul>
  </body>
</html>";

fn make_data() -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("year".to_string(), to_json(&"2015"));

    let mut teams = Vec::new();

    for &(name, score) in &[
        ("Jiangsu", 43u16),
        ("Beijing", 27u16),
        ("Guangzhou", 22u16),
        ("Shandong", 12u16),
    ] {
        let mut t = Map::new();
        t.insert("name".to_string(), to_json(&name));
        t.insert("score".to_string(), to_json(&score));
        teams.push(t)
    }

    data.insert("teams".to_string(), to_json(&teams));
    data
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("table", SOURCE.to_string())
        .expect("Invalid template format");

    let data = make_data();
    b.iter(|| handlebars.render("table", &data).ok().unwrap())
}
