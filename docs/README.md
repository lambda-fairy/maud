# Documentation

This directory contains the documentation for Maud.

## Build

Build the documentation:

    make

The built files will be placed in `site/`.

You can also delete the build artifacts with:

    make clean

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

The documentation is automatically built
and uploaded to GitHub pages
on every commit to `master`.

However,
if you wish to deploy the docs manually,
then you can do that by running `./deploy.sh`.
You will need push access to the Maud repository
to do this.
