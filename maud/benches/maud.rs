#![feature(test)]

use maud::html;

extern crate test;

#[derive(Debug)]
struct Entry {
    name: &'static str,
    score: u16,
}

// Represents a realistic text-heavy workload: rendering user-supplied prose
// (article bodies, comments, descriptions) where the dynamic splice is long
// and goes through the runtime HTML escaper.
#[bench]
fn render_long_text(b: &mut test::Bencher) {
    let body = test::black_box(
        "The quick brown fox jumps over the lazy dog. \
         Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
         eiusmod tempor incididunt ut labore et dolore magna aliqua. \
         A few specials sprinkled in: 5 < 10 & \"quoted\" > none.",
    );
    b.iter(|| {
        html! {
            article {
                p { (body) }
            }
        }
    });
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let year = test::black_box("2015");
    let teams = test::black_box(vec![
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
    });
}
