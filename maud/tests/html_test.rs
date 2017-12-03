#[cfg(test)]
#[macro_export]
macro_rules! html_test {
    (bootstrap: { $($bootstrap:tt)* }, assert_eq: $a_eq:expr, markup: $($m:tt)+) => {
        {
            $($bootstrap)* {
                let s = html! { $($m)+ }.into_string();
                assert_eq!(s, $a_eq, "html!");
            }
        }

        {
            let mut s = String::new();
            $($bootstrap)* {
                html_to! { &mut s, $($m)+ };
                assert_eq!(s, $a_eq, "html_to!, borrowing buffer");
                s.clear();
            }
        }

        {
            let mut s = String::new();
            fn to_borrowed_buffer(buffer: &mut String) {
                $($bootstrap)* {
                    html_to! { buffer, $($m)+ };
                    assert_eq!(buffer, $a_eq, "html_to!, buffer already borrowed");
                    buffer.clear();
                }
            }
            to_borrowed_buffer(&mut s);
        }
    };
    (assert_eq: $a_eq:expr, markup: $($m:tt)+) => {
        html_test!(bootstrap : {}, assert_eq: $a_eq, markup: $($m)+);
    };
}
