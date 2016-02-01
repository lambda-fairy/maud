#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;

use std::fmt;

#[test]
fn literals() {
    let mut s = String::new();
    html!(s, "du\tcks" -23 3.14 '\n' "geese").unwrap();
    assert_eq!(s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let mut s = String::new();
    html!(s, "<flim&flam>").unwrap();
    assert_eq!(s, "&lt;flim&amp;flam&gt;");
}

#[test]
fn semicolons() {
    let mut s = String::new();
    html!(s, {
        "one";
        "two";
        "three";
        ;;;;;;;;;;;;;;;;;;;;;;;;
        "four";
    }).unwrap();
    assert_eq!(s, "onetwothreefour");
}

#[test]
fn blocks() {
    let mut s = String::new();
    html!(s, {
        "hello"
        {
            " ducks" " geese"
        }
        " swans"
    }).unwrap();
    assert_eq!(s, "hello ducks geese swans");
}

mod elements {
    #[test]
    fn simple() {
        let mut s = String::new();
        html!(s, p { b { "pickle" } "barrel" i { "kumquat" } }).unwrap();
        assert_eq!(s, "<p><b>pickle</b>barrel<i>kumquat</i></p>");
    }

    #[test]
    fn nesting() {
        let mut s = String::new();
        html!(s, html body div p sup "butts").unwrap();
        assert_eq!(s, "<html><body><div><p><sup>butts</sup></p></div></body></html>");
    }

    #[test]
    fn empty() {
        let mut s = String::new();
        html!(s, "pinkie" br/ "pie").unwrap();
        assert_eq!(s, "pinkie<br>pie");
    }

    #[test]
    fn attributes() {
        let mut s = String::new();
        html!(s, {
            link rel="stylesheet" href="styles.css"/
            section id="midriff" {
                p class="hotpink" "Hello!"
            }
        }).unwrap();
        assert_eq!(s, concat!(
                r#"<link rel="stylesheet" href="styles.css">"#,
                r#"<section id="midriff"><p class="hotpink">Hello!</p></section>"#));
    }

    #[test]
    fn empty_attributes() {
        let mut s = String::new();
        html!(s, div readonly? input type="checkbox" checked? /).unwrap();
        assert_eq!(s, r#"<div readonly><input type="checkbox" checked></div>"#);
    }
}

mod splices {
    #[test]
    fn literals() {
        let mut s = String::new();
        html!(s, $"<pinkie>").unwrap();
        assert_eq!(s, "&lt;pinkie&gt;");
    }

    #[test]
    fn raw_literals() {
        use maud::PreEscaped;
        let mut s = String::new();
        html!(s, $PreEscaped("<pinkie>")).unwrap();
        assert_eq!(s, "<pinkie>");
    }

    #[test]
    fn blocks() {
        let mut s = String::new();
        html!(s, {
            ${
                let mut result = 1i32;
                for i in 2..11 {
                    result *= i;
                }
                result
            }
        }).unwrap();
        assert_eq!(s, "3628800");
    }

    #[test]
    fn attributes() {
        let rocks = true;
        let mut s = String::new();
        html!(s, {
            input checked?=true /
            input checked?=false /
            input checked?=rocks /
            input checked?=(!rocks) /
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
        html!(s, $BEST_PONY).unwrap();
        assert_eq!(s, "Pinkie Pie");
    }

    #[test]
    fn closures() {
        let best_pony = "Pinkie Pie";
        let mut s = String::new();
        html!(s, $best_pony).unwrap();
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
            "Name: " $pinkie.name ". Rating: " $pinkie.repugnance()
        }).unwrap();
        assert_eq!(s, "Name: Pinkie Pie. Rating: 1");
    }

    #[test]
    fn nested_macro_invocation() {
        let best_pony = "Pinkie Pie";
        let mut s = String::new();
        html!(s, $(format!("{}", best_pony))).unwrap();
        assert_eq!(s, "Pinkie Pie");
    }
}

#[test]
fn issue_13() {
    let owned = String::from("yay");
    let mut s = String::new();
    html!(s, $owned).unwrap();
    let _ = owned;
}

mod control {
    #[test]
    fn if_expr() {
        for (number, &name) in (1..4).zip(["one", "two", "three"].iter()) {
            let mut s = String::new();
            html!(s, {
                #if number == 1 {
                    "one"
                } #else if number == 2 {
                    "two"
                } #else if number == 3 {
                    "three"
                } #else {
                    "oh noes"
                }
            }).unwrap();
            assert_eq!(s, name);
        }
    }

    #[test]
    fn if_let() {
        for &(input, output) in [(Some("yay"), "yay"), (None, "oh noes")].iter() {
            let mut s = String::new();
            html!(s, {
                #if let Some(value) = input {
                    $value
                } #else {
                    "oh noes"
                }
            }).unwrap();
            assert_eq!(s, output);
        }
    }

    #[test]
    fn for_expr() {
        let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"];
        let mut s = String::new();
        html!(s, {
            ul #for pony in &ponies {
                li $pony
            }
        }).unwrap();
        assert_eq!(s, concat!(
                "<ul>",
                "<li>Apple Bloom</li>",
                "<li>Scootaloo</li>",
                "<li>Sweetie Belle</li>",
                "</ul>"));
    }
}

#[test]
fn html_utf8() {
    let mut buf = vec![];
    html_utf8!(buf, p "hello").unwrap();
    assert_eq!(buf, b"<p>hello</p>");
}

mod issue_10 {
    #[test]
    fn hyphens_in_element_names() {
        let mut s = String::new();
        html!(s, custom-element {}).unwrap();
        assert_eq!(s, "<custom-element></custom-element>");
    }

    #[test]
    fn hyphens_in_attribute_names() {
        let mut s = String::new();
        html!(s, this sentence-is="false" of-course? {}).unwrap();
        assert_eq!(s, r#"<this sentence-is="false" of-course></this>"#);
    }
}

#[test]
fn call() {
    fn ducks(w: &mut fmt::Write) -> fmt::Result {
        write!(w, "Ducks")
    }
    let mut s = String::new();
    let swans = |yes|
        if yes {
            |w: &mut fmt::Write| write!(w, "Swans")
        } else {
            panic!("oh noes")
        };
    html!(s, {
        #call ducks
        #call (|w: &mut fmt::Write| write!(w, "Geese"))
        #call swans(true)
    }).unwrap();
    assert_eq!(s, "DucksGeeseSwans");
}

#[test]
fn issue_23() {
    macro_rules! to_string {
        ($($x:tt)*) => {{
            let mut s = String::new();
            html!(s, $($x)*).unwrap();
            s
        }}
    }

    let name = "Lyra";
    let s = to_string!(p { "Hi, " $name "!" });
    assert_eq!(s, "<p>Hi, Lyra!</p>");
}

#[test]
fn tuple_accessors() {
    let mut s = String::new();
    let a = ("ducks", "geese");
    html!(s, { $a.0 }).unwrap();
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
    html!(s, $inner::name()).unwrap();
    assert_eq!(s, "Maud");
}

#[test]
fn class_shorthand() {
    let mut s = String::new();
    html!(s, p { "Hi, " span.name { "Lyra" } "!" }).unwrap();
    assert_eq!(s, "<p>Hi, <span class=\"name\">Lyra</span>!</p>");
}

#[test]
fn div_class_shorthand() {
    let mut s = String::new();
    html!(s, p { "Hi, " .name { "Lyra" } "!" }).unwrap();
    assert_eq!(s, "<p>Hi, <div class=\"name\">Lyra</div>!</p>");
}

#[test]
fn classes_shorthand() {
    let mut s = String::new();
    html!(s, p { "Hi, " span.name.here { "Lyra" } "!" }).unwrap();
    assert_eq!(s, "<p>Hi, <span class=\"name here\">Lyra</span>!</p>");
}

#[test]
fn div_classes_shorthand() {
    let mut s = String::new();
    html!(s, p { "Hi, " .name.here { "Lyra" } "!" }).unwrap();
    assert_eq!(s, "<p>Hi, <div class=\"name here\">Lyra</div>!</p>");
}

#[test]
fn div_classes_shorthand_with_attrs() {
    let mut s = String::new();
    html!(s, p { "Hi, " .name.here id="thing" { "Lyra" } "!" }).unwrap();
    assert_eq!(s, "<p>Hi, <div class=\"name here\" id=\"thing\">Lyra</div>!</p>");
}
