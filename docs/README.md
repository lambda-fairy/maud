# Documentation

This directory contains the documentation for Maud.

It is hosted at <https://maud.lambda.xyz>.

## Build

Build the documentation:

    make

The built files will be placed in `site/`.

You can also delete the build artifacts with:

    make clean

## Style

- [Be brief][short].
- Use [semantic line breaks][sembr].
- American spelling.
- Parentheticals should use spaced en dashes –
  like this –
  not em dashes.

[short]: https://developers.google.com/tech-writing/one/short-sentences
[sembr]: https://sembr.org/

## Watch

To ease editing,
there is a `./watch.sh` script
that starts a web server
and rebuilds the site
on every change.

The script uses Python 3 and [entr],
but feel free to adapt it
to your environment.

[entr]: http://eradman.com/entrproject/

## Add a new page

The list of pages to be built
is defined by the `slugs` variable
in the Makefile.

To add a new page,
create a Markdown file in `content/`
and add its name
(excluding the `.md`)
to `slugs`:

```diff
-slugs := index getting-started basic-syntax partials ...
+slugs := index getting-started basic-syntax my-awesome-new-page partials ...
```

The order of the names in `slugs`
determines the order of entries
in the table of contents.

Make sure that
your page starts with a heading
(e.g. `# My awesome new page`)
or it won't show up.

## The navigation cache (`nav.json`)

The site generator
constructs the table of contents dynamically
by reading each Markdown page
and extracting its heading.
This data is cached in `nav.json`.

You don't need to care about this file
most of the time,
but it might be useful to know about it
when hacking on the site generator itself.

## Deployment

The documentation is built
and uploaded to GitHub Pages
using [GitHub Actions][config].

The workflow is run automatically on a new release.
For changes not tied to a release (e.g. typo fixes),
a maintainer can trigger it manually –
please ask if you'd like this.

[config]: ../.github/workflows/publish-docs.yml
