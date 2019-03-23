MARKDOWN_FILES := $(wildcard content/*.md)
HTML_FILES := $(patsubst content/%.md,site/%.html,$(MARKDOWN_FILES))

.PHONY: all
all: $(HTML_FILES) site/styles.css

target/debug/docs: $(wildcard src/*)
	cargo build

site/%.html: content/%.md target/debug/docs
	target/debug/docs build-page $< $@

site/styles.css: styles.css
	cp $^ $@

.PHONY: clean
clean:
	rm -fr site
