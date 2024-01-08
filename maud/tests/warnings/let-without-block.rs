use maud::html;

fn main() {
    html! {
        p.@let x = 1; {
            (x)
        }
    };
}
