#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;

#[test]
fn literals() {
    let mut s = String::new();
    html!(s, ("<pinkie>")).unwrap();
    assert_eq!(s, "&lt;pinkie&gt;");
}

#[test]
fn raw_literals() {
    use maud::PreEscaped;
    let mut s = String::new();
    html!(s, (PreEscaped("<pinkie>"))).unwrap();
    assert_eq!(s, "<pinkie>");
}

#[test]
fn blocks() {
    let mut s = String::new();
    html!(s, {
        ({
            let mut result = 1i32;
            for i in 2..11 {
                result *= i;
            }
            result
        })
    }).unwrap();
    assert_eq!(s, "3628800");
}

#[test]
fn attributes() {
    let alt = "Pinkie Pie";
    let mut s = String::new();
    html!(s, img src="pinkie.jpg" alt=(alt) /).unwrap();
    assert_eq!(s, r#"<img src="pinkie.jpg" alt="Pinkie Pie">"#);
}

#[test]
fn empty_attributes() {
    let rocks = true;
    let mut s = String::new();
    html!(s, {
        input checked?(true) /
        input checked?(false) /
        input checked?(rocks) /
        input checked?(!rocks) /
    }).unwrap();
    assert_eq!(s, concat!(
            r#"<input checked>"#,
            r#"<input>"#,
            r#"<input checked>"#,
            r#"<input>"#));
}

static BEST_PONY: &'static str = "Pinkie Pie";

#[test]
fn statics() {
    let mut s = String::new();
    html!(s, (BEST_PONY)).unwrap();
    assert_eq!(s, "Pinkie Pie");
}

#[test]
fn locals() {
    let best_pony = "Pinkie Pie";
    let mut s = String::new();
    html!(s, (best_pony)).unwrap();
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
    let mut s = String::new();
    html!(s, {
        "Name: " (pinkie.name) ". Rating: " (pinkie.repugnance())
    }).unwrap();
    assert_eq!(s, "Name: Pinkie Pie. Rating: 1");
}

#[test]
fn tuple_accessors() {
    let mut s = String::new();
    let a = ("ducks", "geese");
    html!(s, (a.0)).unwrap();
    assert_eq!(s, "ducks");
}

#[test]
fn splice_with_path() {
    mod inner {
        pub fn name() -> &'static str {
            "Maud"
        }
    }
    let mut s = String::new();
    html!(s, (inner::name())).unwrap();
    assert_eq!(s, "Maud");
}

#[test]
fn nested_macro_invocation() {
    let best_pony = "Pinkie Pie";
    let mut s = String::new();
    html!(s, (format!("{}", best_pony))).unwrap();
    assert_eq!(s, "Pinkie Pie");
}
