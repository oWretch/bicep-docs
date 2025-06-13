# Makefile for generating Bicep documentation examples
# This file demonstrates various output formats and options available in bicep-docs

# Variables
BICEP_FILE = example.bicep
BICEP_DOCS = ../../target/debug/bicep-docs

# Build the binary first
$(BICEP_DOCS):
	cd ../.. && cargo build

# Default target - generate all examples
.PHONY: all
all: markdown asciidoc yaml json clean-temp

# Clean all generated files
.PHONY: clean
clean:
	rm -f output-*.md output-*.adoc output-*.yaml output-*.json

# Clean temporary files
.PHONY: clean-temp
clean-temp:
	rm -f temp-*

# Markdown Examples
.PHONY: markdown
markdown: $(BICEP_DOCS) markdown-default markdown-emoji markdown-exclude-empty markdown-all-options

markdown-default: $(BICEP_DOCS)
	$(BICEP_DOCS) markdown $(BICEP_FILE) -o output-markdown-default.md

markdown-emoji: $(BICEP_DOCS)
	$(BICEP_DOCS) markdown $(BICEP_FILE) --emoji -o output-markdown-emoji.md

markdown-exclude-empty: $(BICEP_DOCS)
	$(BICEP_DOCS) markdown $(BICEP_FILE) --exclude-empty -o output-markdown-exclude-empty.md

markdown-all-options: $(BICEP_DOCS)
	$(BICEP_DOCS) markdown $(BICEP_FILE) --emoji --exclude-empty -o output-markdown-all-options.md

# AsciiDoc Examples
.PHONY: asciidoc
asciidoc: $(BICEP_DOCS) asciidoc-default asciidoc-emoji asciidoc-exclude-empty asciidoc-all-options

asciidoc-default: $(BICEP_DOCS)
	$(BICEP_DOCS) asciidoc $(BICEP_FILE) -o output-asciidoc-default.adoc

asciidoc-emoji: $(BICEP_DOCS)
	$(BICEP_DOCS) asciidoc $(BICEP_FILE) --emoji -o output-asciidoc-emoji.adoc

asciidoc-exclude-empty: $(BICEP_DOCS)
	$(BICEP_DOCS) asciidoc $(BICEP_FILE) --exclude-empty -o output-asciidoc-exclude-empty.adoc

asciidoc-all-options: $(BICEP_DOCS)
	$(BICEP_DOCS) asciidoc $(BICEP_FILE) --emoji --exclude-empty -o output-asciidoc-all-options.adoc

# YAML Examples
.PHONY: yaml
yaml: $(BICEP_DOCS) yaml-default yaml-exclude-empty

yaml-default: $(BICEP_DOCS)
	$(BICEP_DOCS) yaml $(BICEP_FILE) -o output-yaml-default.yaml

yaml-exclude-empty: $(BICEP_DOCS)
	$(BICEP_DOCS) yaml $(BICEP_FILE) --exclude-empty -o output-yaml-exclude-empty.yaml

# JSON Examples
.PHONY: json
json: $(BICEP_DOCS) json-default json-pretty json-exclude-empty json-all-options

json-default: $(BICEP_DOCS)
	$(BICEP_DOCS) json $(BICEP_FILE) -o output-json-default.json

json-pretty: $(BICEP_DOCS)
	$(BICEP_DOCS) json $(BICEP_FILE) --pretty -o output-json-pretty.json

json-exclude-empty: $(BICEP_DOCS)
	$(BICEP_DOCS) json $(BICEP_FILE) --exclude-empty -o output-json-exclude-empty.json

json-all-options: $(BICEP_DOCS)
	$(BICEP_DOCS) json $(BICEP_FILE) --pretty --exclude-empty -o output-json-all-options.json

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all          - Generate all example outputs"
	@echo "  clean        - Remove all generated files"
	@echo ""
	@echo "Format-specific targets:"
	@echo "  markdown     - Generate all Markdown examples"
	@echo "  asciidoc     - Generate all AsciiDoc examples"
	@echo "  yaml         - Generate all YAML examples"
	@echo "  json         - Generate all JSON examples"
	@echo ""
	@echo "Individual examples:"
	@echo "  markdown-default, markdown-emoji, markdown-exclude-empty, markdown-all-options"
	@echo "  asciidoc-default, asciidoc-emoji, asciidoc-exclude-empty, asciidoc-all-options"
	@echo "  yaml-default, yaml-exclude-empty"
	@echo "  json-default, json-pretty, json-exclude-empty, json-all-options"
	@echo ""
	@echo "Source file: $(BICEP_FILE)"
