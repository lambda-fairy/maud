use maud::html;

fn main() {
    let name = "href";
    html! {
        a (name)="about:blank" {}
    };
}
