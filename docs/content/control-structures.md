# Control structures

Maud provides various control structures for adding dynamic elements to your templates.

## Branching with `@if` and `@else`

Use `@if` and `@else` to branch on a boolean expression. As with Rust, braces are mandatory and the `@else` clause is optional.

```rust
#[derive(PartialEq)]
enum Princess { Celestia, Luna, Cadance, TwilightSparkle }

let user = Princess::Celestia;

html! {
    @if user == Princess::Luna {
        h1 { "Super secret woona to-do list" }
        ul {
            li { "Nuke the Crystal Empire" }
            li { "Kick a puppy" }
            li { "Evil laugh" }
        }
    } @else if user == Princess::Celestia {
        p { "Sister, please stop reading my diary. It's rude." }
    } @else {
        p { "Nothing to see here; move along." }
    }
}
```

`@if let` is supported as well. It works as you'd expect:

```rust
let user = Some("Pinkie Pie");
html! {
    p {
        "Hello, "
        @if let Some(name) = user {
            (name)
        } @else {
            "stranger"
        }
        "!"
    }
}
```

## Looping with `@for`

Use `@for .. in ..` to loop over the elements of an iterator.

```rust
let names = ["Applejack", "Rarity", "Fluttershy"];
html! {
    p { "My favorite ponies are:" }
    ol {
        @for name in &names {
            li { (name) }
        }
    }
}
```

## Declaring variables with `@let`

Declare a new variable within a template using `@let`. This can be useful when working with values in a for loop.

```rust
let names = ["Applejack", "Rarity", "Fluttershy"];
html! {
    @for name in &names {
        @let first_letter = name.chars().next().unwrap();
        p {
            "The first letter of "
            b { (name) }
            " is "
            b { (first_letter) }
            "."
        }
    }
}
```

## Matching with `@match`

Pattern matching is supported with `@match`.

```rust
enum Princess { Celestia, Luna, Cadance, TwilightSparkle }

let user = Princess::Celestia;

html! {
    @match user {
        Princess::Luna => {
            h1 { "Super secret woona to-do list" }
            ul {
                li { "Nuke the Crystal Empire" }
                li { "Kick a puppy" }
                li { "Evil laugh" }
            }
        },
        Princess::Celestia => {
            p { "Sister, please stop reading my diary. It's rude." }
        },
        _ => p { "Nothing to see here; move along." }
    }
}
```
