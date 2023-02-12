# Frequently asked questions

## What is the origin of the name "Maud"?

Maud is named after a [character] from *My Little Pony: Friendship is Magic*.
It does not refer to the [poem] by Alfred Tennyson, though other people have brought that up in the past.

Here are some reasons why I chose this name:

* "Maud" shares three letters with "markup";

* The library is efficient and austere, like the character;

* Google used to maintain a site called ["HTML5 Rocks"], and Maud (the character) is a geologist.

[character]: http://mlp.wikia.com/wiki/Maud_Pie
[poem]: https://en.wikipedia.org/wiki/Maud_and_other_poems
["HTML5 Rocks"]: https://techcrunch.com/2010/06/22/html5rocks-google/

## Why does `html!` always allocate a `String`? Wouldn't it be more efficient if it wrote to a handle directly?

Good question! In fact, Maud did work this way in the past.

But it's hard to support buffer reuse in an ergonomic way.
The approaches I tried either involved too much boilerplate, or caused mysterious lifetime issues, or both.
Moreover, Maud's allocation pattern—with small, short-lived buffers—follow the fast path in modern allocators.
These reasons are why I changed `html!` to return a `String` in version 0.11.

That said, Rust has changed a lot since then, and some of those old assumptions might no longer hold today.
So this decision could be revisited prior to the 1.0 release.

## Why is Maud written as a procedural macro? Can't it use `macro_rules!` instead?

This is certainly possible, and indeed the [Horrorshow] library works this way.

I use procedural macros because they are more flexible.
There are some syntax constructs in Maud that are hard to parse with `macro_rules!`; better diagnostics are a bonus as well.

[Horrorshow]: https://github.com/Stebalien/horrorshow-rs

## Maud has had a lot of releases so far. When will it reach 1.0?

I originally planned to cut a 1.0 after implementing stable support.
But now that's happened, I've realized that there are a couple design questions that I'd like to resolve before marking that milestone.
Expect a blog post on this topic Very Soon®.

## Why doesn't Maud implement [context-aware escaping]?

I agree that context-aware escaping is very important, especially for the kind of small-scale development that Maud is used for.
But it's a complex feature, with security implications, so I want to take the time to get it right.

Please follow [#181] for the latest developments!

[context-aware escaping]: https://security.googleblog.com/2009/03/reducing-xss-by-way-of-automatic.html
[#181]: https://github.com/lambda-fairy/maud/issues/181
