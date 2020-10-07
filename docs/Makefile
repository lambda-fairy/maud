slugs := index getting-started basic-syntax dynamic-content partials control-structures traits web-frameworks faq

slug_to_md = content/$(1).md
slug_to_html = site/$(1).html

builder := target/debug/docs
nav_json := site/nav.json

version := $(shell git describe)
hash := $(shell git rev-parse HEAD)

md_files := $(foreach slug,$(slugs),$(call slug_to_md,$(slug)))
html_files := $(foreach slug,$(slugs),$(call slug_to_html,$(slug)))
slugs_and_md_files := $(foreach slug,$(slugs),$(slug):$(call slug_to_md,$(slug)))

print_status = @ printf ' \e[1;35m♦ %s\e[0m\n' '$(1)'

.PHONY: all
all: $(html_files) site/styles.css

$(builder): $(wildcard src/*)
	$(call print_status,Cargo)
	@ cargo build --locked

$(nav_json): $(md_files) $(builder)
	$(call print_status,Table of contents)
	@ $(builder) build-nav $@ $(slugs_and_md_files)

site/%.html: content/%.md $(nav_json) $(builder)
	$(call print_status,Render $(*F))
	@ $(builder) build-page $@ $(*F) $< $(nav_json) $(version) $(hash)

site/styles.css: styles.css
	$(call print_status,Copy stylesheet)
	@ mkdir -p $(dir $@)
	@ cp $^ $@

.PHONY: clean
clean:
	$(call print_status,Clean)
	@ cargo clean
	@ rm -fr site
