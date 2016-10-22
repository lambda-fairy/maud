#![feature(proc_macro, test)]

#[macro_use]
extern crate serde_derive;
extern crate tera;
extern crate test;

use tera::{Context, Tera};

#[derive(Serialize, Debug)]
struct Entry {
    name: &'static str,
    score: u16,
}

static SOURCE: &'static str = "<html>
  <head>
    <title>{{ year }}</title>
  </head>
  <body>
    <h1>CSL {{ year }}</h1>
    <ul>
    {% for team in teams %}
      <li class=\"{% if loop.first %}champion{% endif %}\">
      <b>{{ team.name }}</b>: {{ team.score }}
      </li>
    {% endfor %}
    </ul>
  </body>
</html>";

#[bench]
fn render_template(b: &mut test::Bencher) {
    let mut tera = test::black_box(Tera::default());
    tera.add_template("table", SOURCE);

    let context = test::black_box({
        let mut context = Context::new();
        context.add("teams", &[
            Entry { name: "Jiangsu", score: 43 },
            Entry { name: "Beijing", score: 27 },
            Entry { name: "Guangzhou", score: 22 },
            Entry { name: "Shandong", score: 12 },
        ]);
        context.add("year", &"2015");
        context
    });

    // FIXME: is there a way to avoid this clone?
    b.iter(|| tera.render("table", context.clone()));
}
