use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use maud::html;

#[derive(Debug)]
struct Entry {
    name: &'static str,
    score: u16,
}

fn render_template(c: &mut Criterion) {
    c.bench_function("render_template", |b: &mut Bencher| {
        let year = black_box("2015");
        let teams = black_box(vec![
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
        ]);

        b.iter(|| {
            html! {
                html {
                    head {
                        title { (year) }
                    }
                    body {
                        h1 { "CSL " (year) }
                        ul {
                            @for (i, team) in teams.iter().enumerate() {
                                li.champion[i == 0] {
                                    b { (team.name) ": " (team.score) }
                                }
                            }
                        }
                    }
                }
            }
        })
    });
}

criterion_group!(benches, render_template);
criterion_main!(benches);
