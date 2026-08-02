use maud::html;

fn main() {
    html! {
        div;
        p;
        my-custom-element;
        .thing;
        @if true {
            span;
        }
    };
}
