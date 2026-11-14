use maud::html;

fn main() {
    html! {
        br {}
        img alt="some text" {}
        meta charset="utf-8" {}
        div {
            a { hr {} }
        }
    };
}
