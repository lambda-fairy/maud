use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use maud::{html, Markup};

#[derive(Debug)]
struct Entry {
    name: &'static str,
    score: u32,
}

mod btn {
    use maud::{html, Markup, Render};

    #[derive(Copy, Clone)]
    pub enum RequestMethod {
        Get,
        Post,
    }

    #[derive(Copy, Clone)]
    pub struct Button<'a> {
        label: &'a str,
        path: &'a str,
        req_meth: RequestMethod,
    }

    impl<'a> Button<'a> {
        pub fn new(label: &'a str, path: &'a str) -> Button<'a> {
            Button {
                label,
                path,
                req_meth: RequestMethod::Get,
            }
        }

        pub fn with_method(mut self, meth: RequestMethod) -> Button<'a> {
            self.req_meth = meth;
            self
        }
    }

    impl<'a> Render for Button<'a> {
        fn render(&self) -> Markup {
            match self.req_meth {
                RequestMethod::Get => {
                    html! { a.btn href=(self.path) { (self.label) } }
                }
                RequestMethod::Post => {
                    html! {
                        form method="POST" action=(self.path) {
                            input.btn type="submit" value=(self.label);
                        }
                    }
                }
            }
        }
    }
}

fn layout<S: AsRef<str>>(title: S, inner: Markup) -> Markup {
    html! {
        html {
            head {
                title { (title.as_ref()) }
            }
            body {
                (inner)
            }
        }
    }
}

fn render_complicated_template(c: &mut Criterion) {
    c.bench_function("render_complicated_template", |b: &mut Bencher| {
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
            use crate::btn::{Button, RequestMethod};
            layout(
                format!("Homepage of {}", year),
                html! {
                    h1 { "Hello there!" }

                    @for entry in &teams {
                        div {
                            strong { (entry.name) ": " (entry.score) }
                            (Button::new("Edit", "edit"))
                            (Button::new("Delete", "edit")
                                        .with_method(RequestMethod::Post))
                        }
                    }
                },
            )
        });
    });
}

criterion_group!(benches, render_complicated_template);
criterion_main!(benches);
