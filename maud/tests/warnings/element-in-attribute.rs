use maud::html;

fn main() {
    html! {
        a href={ b {} } {}

        a .{ b {} } {}
        a #{ b {} } {}

        @if true {
            a href={ b #if {} } {}
        } @else if true {
            a href={ b #else-if {} } {}
        } @else {
            a href={ b #else {} } {}
        }

        @for _ in 0..10 {
            a href={ b #for {} } {}
        }

        @while false {
            a href={ b #while {} } {}
        }

        @match () {
            () => a href={ b #match {} } {}
        }
    };
}
