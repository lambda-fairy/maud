use maud::html;

fn main() {
    html! {
        a href={ div {} } {}
    };

    html! {
        a .{ div {} } {}
    };

    html! {
        a #{ div {} } {}
    };

    html! {
        @if true {
            a href={ div {} } {}
        }
    };
}
