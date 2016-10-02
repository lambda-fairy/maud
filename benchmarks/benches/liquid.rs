#![feature(test)]

extern crate liquid;
extern crate test;

use liquid::{Context, Renderable, Value};
use std::collections::HashMap;

// FIXME(cobalt-org/liquid-rust#47): "for_loop" should be "forloop" instead
static SOURCE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {% for team in teams %}
      <li class=\"{% if for_loop.first %}champion{% endif %}\">
      <b>{{team.name}}</b>: {{team.score}}
      </li>
    {% endfor %}
    </ul>
  </body>
</html>";

fn make_team(name: &'static str, score: u16) -> Value {
    let mut team = HashMap::new();
    team.insert("name".to_string(), Value::Str(name.to_string()));
    team.insert("score".to_string(), Value::Num(score as f32));
    Value::Object(team)
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let template = test::black_box(liquid::parse(SOURCE, Default::default()).unwrap());
    let mut context = test::black_box({
        let mut context = Context::new();
        context.set_val("year", Value::Num(2015.));
        let teams = vec![
            make_team("Jiangsu", 43),
            make_team("Beijing", 27),
            make_team("Guangzhou", 22),
            make_team("Shandong", 12),
        ];
        context.set_val("teams", Value::Array(teams));
        context
    });
    b.iter(|| template.render(&mut context).unwrap().unwrap());
}
