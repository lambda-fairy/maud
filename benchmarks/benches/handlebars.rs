// https://github.com/sunng87/handlebars-rust/blob/master/benches/bench.rs

#![feature(test)]

extern crate handlebars;
extern crate rustc_serialize as serialize;
extern crate test;

use std::collections::BTreeMap;

use handlebars::Handlebars;
use serialize::json::{Json, ToJson};

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

fn make_data() -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    data.insert("year".to_string(), "2015".to_json());

    let mut teams = Vec::new();

    for v in vec![("Jiangsu", 43u16),
                  ("Beijing", 27u16),
                  ("Guangzhou", 22u16),
                  ("Shandong", 12u16)]
                 .iter() {
        let (name, score) = *v;
        let mut t = BTreeMap::new();
        t.insert("name".to_string(), name.to_json());
        t.insert("score".to_string(), score.to_json());
        teams.push(t)
    }

    data.insert("teams".to_string(), teams.to_json());
    data
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("table", SOURCE.to_string())
              .ok()
              .expect("Invalid template format");

    let data = make_data();
    b.iter(|| handlebars.render("table", &data).ok().unwrap())
}
