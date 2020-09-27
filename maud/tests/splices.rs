use maud::html;

#[test]
fn literals() {
    let s = html!(("<pinkie>")).into_string();
    assert_eq!(s, "&lt;pinkie&gt;");
}

#[test]
fn raw_literals() {
    use maud::PreEscaped;
    let s = html!((PreEscaped("<pinkie>"))).into_string();
    assert_eq!(s, "<pinkie>");
}

#[test]
fn blocks() {
    let s = html!({
        ({
            let mut result = 1i32;
            for i in 2..11 {
                result *= i;
            }
            result
        })
    })
    .into_string();
    assert_eq!(s, "3628800");
}

#[test]
fn attributes() {
    let alt = "Pinkie Pie";
    let s = html!(img src="pinkie.jpg" alt=(alt) /).into_string();
    assert_eq!(s, r#"<img src="pinkie.jpg" alt="Pinkie Pie">"#);
}

#[test]
fn class_shorthand() {
    let pinkie_class = "pinkie";
    let s = html!(p.(pinkie_class) { "Fun!" }).into_string();
    assert_eq!(s, r#"<p class="pinkie">Fun!</p>"#);
}

#[test]
fn class_shorthand_block() {
    let class_prefix = "pinkie-";
    let s = html!(p.{ (class_prefix) "123" } { "Fun!" }).into_string();
    assert_eq!(s, r#"<p class="pinkie-123">Fun!</p>"#);
}

#[test]
fn id_shorthand() {
    let pinkie_id = "pinkie";
    let s = html!(p#(pinkie_id) { "Fun!" }).into_string();
    assert_eq!(s, r#"<p id="pinkie">Fun!</p>"#);
}

static BEST_PONY: &'static str = "Pinkie Pie";

#[test]
fn statics() {
    let s = html!((BEST_PONY)).into_string();
    assert_eq!(s, "Pinkie Pie");
}

#[test]
fn locals() {
    let best_pony = "Pinkie Pie";
    let s = html!((best_pony)).into_string();
    assert_eq!(s, "Pinkie Pie");
}

/// An example struct, for testing purposes only
struct Creature {
    name: &'static str,
    /// Rating out of 10, where:
    /// * 0 is a naked mole rat with dysentery
    /// * 10 is Sweetie Belle in a milkshake
    adorableness: u32,
}

impl Creature {
    fn repugnance(&self) -> u32 {
        10 - self.adorableness
    }
}

#[test]
fn structs() {
    let pinkie = Creature {
        name: "Pinkie Pie",
        adorableness: 9,
    };
    let s = html!({
        "Name: " (pinkie.name) ". Rating: " (pinkie.repugnance())
    })
    .into_string();
    assert_eq!(s, "Name: Pinkie Pie. Rating: 1");
}

#[test]
fn tuple_accessors() {
    let a = ("ducks", "geese");
    let s = html!((a.0)).into_string();
    assert_eq!(s, "ducks");
}

#[test]
fn splice_with_path() {
    mod inner {
        pub fn name() -> &'static str {
            "Maud"
        }
    }
    let s = html!((inner::name())).into_string();
    assert_eq!(s, "Maud");
}

#[test]
fn nested_macro_invocation() {
    let best_pony = "Pinkie Pie";
    let s = html!((format!("{} is best pony", best_pony))).into_string();
    assert_eq!(s, "Pinkie Pie is best pony");
}
