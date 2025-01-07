use maud::html;

fn main() {
    html! {
        a href={ b {} } {}
    };

    html! {
        a href=.pinkie-pie {} {}
    };

    html! {
        a .{ b {} } {}
    };

    html! {
        a #{ b {} } {}
    };

    html! {
        @if true {
        } @else if true {
        } @else {
            a href={ b #if-else {} } {}
        }
    };

    html! {
        @for _ in 0..10 {
            a href={ b #for {} } {}
        }
    };

    html! {
        @while false {
            a href={ b #while {} } {}
        }
    };

    html! {
        @match () {
            () => a href={ b #match {} } {}
        }
    };
}
