#![feature(test)]

extern crate test;

use liquid::ParserBuilder;
use liquid::value::{Object, Value};

static SOURCE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {% for team in teams %}
      <li class=\"{% if forloop.first %}champion{% endif %}\">
      <b>{{team.name}}</b>: {{team.score}}
      </li>
    {% endfor %}
    </ul>
  </body>
</html>";

fn make_team(name: &'static str, score: u16) -> Value {
    let mut team = Object::new();
    team.insert("name".into(), Value::scalar(name));
    team.insert("score".into(), Value::scalar(score as i32));
    Value::Object(team)
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let template = test::black_box(ParserBuilder::with_liquid().build().unwrap().parse(SOURCE).unwrap());
    let mut globals = test::black_box({
        let mut globals = Object::new();
        globals.insert("year".into(), Value::scalar(2015));
        let teams = vec![
            make_team("Jiangsu", 43),
            make_team("Beijing", 27),
            make_team("Guangzhou", 22),
            make_team("Shandong", 12),
        ];
        globals.insert("teams".into(), Value::Array(teams));
        globals
    });
    b.iter(|| template.render(&mut globals).unwrap());
}
