slugs := index getting-started text-escaping elements-attributes splices-toggles control-structures partials render-trait web-frameworks faq

slug_to_md = content/$(1).md
slug_to_html = site/$(1).html

build_nav := target/debug/build_nav
build_page := target/debug/build_page

nav_json := site/nav.json

version := $(shell git describe --tags)
hash := $(shell git rev-parse HEAD)

md_files := $(foreach slug,$(slugs),$(call slug_to_md,$(slug)))
html_files := $(foreach slug,$(slugs),$(call slug_to_html,$(slug)))
slugs_and_md_files := $(foreach slug,$(slugs),$(slug):$(call slug_to_md,$(slug)))

print_status = @ printf ' \033[1;35m♦ %s\033[0m\n' '$(1)'

.PHONY: all
all: $(html_files) site/styles.css

$(build_nav) $(build_page): target/debug/%: $(shell find src)
	$(call print_status,Cargo $(@F))
	@ cargo build --bin $(@F) --locked

$(nav_json): $(md_files) $(build_nav)
	$(call print_status,Table of contents)
	@ $(build_nav) $@ $(slugs_and_md_files)

site/%.html: content/%.md $(nav_json) $(build_page)
	$(call print_status,Render $(*F))
	@ $(build_page) $@ $(*F) $< $(nav_json) $(version) $(hash)

site/styles.css: styles.css
	$(call print_status,Copy stylesheet)
	@ mkdir -p $(dir $@)
	@ cp $^ $@

.PHONY: clean
clean:
	$(call print_status,Clean)
	@ cargo clean
	@ rm -fr site
