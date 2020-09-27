#![feature(test)]

extern crate test;

use askama::Template;

#[derive(Template)]
#[template(
    source = r#"
<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {% for team in teams %}
      <li class="{% if loop.index == 1 %}champion{% endif %}">
      <b>{{team.name}}</b>: {{team.score}}
      </li>
    {% endfor %}
    </ul>
  </body>
</html>"#,
    ext = "html"
)]

struct BenchTemplate {
    year: &'static str,
    teams: Vec<Entry>,
}

struct Entry {
    name: &'static str,
    score: u16,
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let teams = vec![
        Entry {
            name: "Jiangsu",
            score: 43,
        },
        Entry {
            name: "Beijing",
            score: 27,
        },
        Entry {
            name: "Guangzhou",
            score: 22,
        },
        Entry {
            name: "Shandong",
            score: 12,
        },
    ];
    let hello = test::black_box(BenchTemplate {
        year: "2015",
        teams,
    });
    b.iter(|| {
        // Instead of simply call hello.render().unwrap(), rendering to
        // a string with a good capacity gives a ~10% speed increase here
        let mut s = String::with_capacity(500);
        hello.render_into(&mut s).unwrap();
    });
}
