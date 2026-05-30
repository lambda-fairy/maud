use maud::html;

fn main() {
    html! {
        42
        42usize
        42.0
        'a'
        b"a"
        b'a'

        // `true` and `false` are only considered literals in attribute values
        input disabled=true;
        input disabled=false;
    };
}
