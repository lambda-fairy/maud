#![feature(test)]

// TODO https://github.com/Stebalien/horrorshow-rs/issues/26
#[macro_use]
extern crate horrorshow;
extern crate test;

use horrorshow::prelude::*;

#[derive(Debug)]
struct Entry {
    name: &'static str,
    score: u16,
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let year = test::black_box("2015");
    let teams = test::black_box(vec![
        Entry { name: "Jiangsu", score: 43 },
        Entry { name: "Beijing", score: 27 },
        Entry { name: "Guangzhou", score: 22 },
        Entry { name: "Shandong", score: 12 },
    ]);
    b.iter(|| {
        (html! {

            html {
                head {
                    title : year;
                }
                body {
                    h1 { : "CSL "; : year }
                    ul {
                        @ for (i, team) in teams.iter().enumerate() {
                            li(class=(if i == 0 { "champion" } else { "" })) {
                                b : (team.name);
                                : ": ";
                                : (team.score);
                            }
                        }
                    }
                }
            }
        }).into_string().unwrap()
    });
}
