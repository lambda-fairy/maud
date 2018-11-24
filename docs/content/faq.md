# Frequently asked questions

## What is the origin of the name "Maud"?

Maud is named after a [character](http://mlp.wikia.com/wiki/Maud_Pie) from *My Little Pony: Friendship is Magic*. It does not refer to the [poem](https://en.wikipedia.org/wiki/Maud_and_other_poems) by Alfred Tennyson, though other people have brought that up in the past.

Here are some reasons why I chose this name:

* "Maud" shares three letters with "markup";

* The library is efficient and austere, like the character;

* Google used to maintain a site called ["HTML5 Rocks"](https://techcrunch.com/2010/06/22/html5rocks-google/), and Maud (the character) is a geologist.

## Why does `html!` always allocate a `String`? Wouldn't it be more efficient if it wrote to a handle directly?

Good question! In fact, Maud did work this way in the past.

Sadly, that kind of thing didn't work out that well in practice. Having to pass the handle around made templates hard to compose, which is important in any non-trivial project. Furthermore, Iron (and other middleware frameworks) likes to take ownership of the response body, so we'd need to do some closure gymnastics to get everything to work together. To put the nail in the coffin, benchmarks showed that a `String` based solution is actually faster than one which avoids allocations.

For these reasons, I changed `html!` to return a `String` in version 0.11.

## Why is Maud written as a procedural macro? Can't it use `macro_rules!` instead?

This is certainly possible, and in fact the [Horrorshow](https://github.com/Stebalien/horrorshow-rs) library works this way.

I use procedural macros because they are more flexible. There are some syntax constructs in Maud that cannot be parsed with `macro_rules!`; better diagnostics are a bonus as well.

## Maud has had a lot of releases so far. When will it reach 1.0?

I plan to make a 1.0 release when the library can be used on stable Rust.

## Why doesn't Maud implement [context-aware escaping](https://security.googleblog.com/2009/03/reducing-xss-by-way-of-automatic.html)?

If a project follows best practices in separating HTML and CSS/JavaScript, then context-aware escaping is unnecessary.

Google uses context-aware escaping because it has a large, decades-old code base, much of it written before these best practices were well known. Any project that uses Maud is neither large nor decades-old, and so should not have the need for this feature.
