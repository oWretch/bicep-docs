# Top-level Makefile for Bicep-Docs Examples
# This file coordinates all example subdirectories and their Makefiles

# Variables
BICEP_DOCS = ../target/debug/bicep-docs
EXAMPLE_DIRS = single-file

# Build the binary first
$(BICEP_DOCS):
	cd .. && cargo build

# Default target - generate all examples in all directories
.PHONY: all
all: $(BICEP_DOCS)
	@echo "=== Generating all examples ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Building examples in $$dir..."; \
			$(MAKE) -C $$dir all; \
		fi; \
	done

# Clean all generated files in all directories
.PHONY: clean
clean:
	@echo "=== Cleaning all examples ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Cleaning examples in $$dir..."; \
			$(MAKE) -C $$dir clean; \
		fi; \
	done

# Format-specific targets that run across all directories
.PHONY: markdown
markdown: $(BICEP_DOCS)
	@echo "=== Generating Markdown examples ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Building Markdown examples in $$dir..."; \
			$(MAKE) -C $$dir markdown; \
		fi; \
	done

.PHONY: asciidoc
asciidoc: $(BICEP_DOCS)
	@echo "=== Generating AsciiDoc examples ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Building AsciiDoc examples in $$dir..."; \
			$(MAKE) -C $$dir asciidoc; \
		fi; \
	done

.PHONY: yaml
yaml: $(BICEP_DOCS)
	@echo "=== Generating YAML examples ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Building YAML examples in $$dir..."; \
			$(MAKE) -C $$dir yaml; \
		fi; \
	done

.PHONY: json
json: $(BICEP_DOCS)
	@echo "=== Generating JSON examples ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Building JSON examples in $$dir..."; \
			$(MAKE) -C $$dir json; \
		fi; \
	done

# Convenience aliases
.PHONY: md
md: markdown

.PHONY: adoc
adoc: asciidoc

# Directory-specific targets
.PHONY: single-file
single-file: $(BICEP_DOCS)
	@echo "=== Building single-file examples ==="
	$(MAKE) -C single-file all

.PHONY: single-file-clean
single-file-clean:
	@echo "=== Cleaning single-file examples ==="
	$(MAKE) -C single-file clean

# Specific format targets for each directory
.PHONY: single-file-markdown
single-file-markdown: $(BICEP_DOCS)
	$(MAKE) -C single-file markdown

.PHONY: single-file-asciidoc
single-file-asciidoc: $(BICEP_DOCS)
	$(MAKE) -C single-file asciidoc

.PHONY: single-file-yaml
single-file-yaml: $(BICEP_DOCS)
	$(MAKE) -C single-file yaml

.PHONY: single-file-json
single-file-json: $(BICEP_DOCS)
	$(MAKE) -C single-file json

# List all available examples
.PHONY: list
list:
	@echo "Available example directories:"
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -d $$dir ]; then \
			echo "  - $$dir"; \
			if [ -f $$dir/README.adoc ]; then \
				echo "    Documentation: $$dir/README.adoc"; \
			fi; \
		fi; \
	done

# Show status of all examples
.PHONY: status
status:
	@echo "=== Example Status ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -d $$dir ]; then \
			echo "Directory: $$dir"; \
			if [ -f $$dir/Makefile ]; then \
				echo "  ✓ Has Makefile"; \
			else \
				echo "  ✗ Missing Makefile"; \
			fi; \
			if [ -f $$dir/README.adoc ]; then \
				echo "  ✓ Has README.adoc"; \
			else \
				echo "  ✗ Missing README.adoc"; \
			fi; \
			output_count=$$(find $$dir -name "output-*" 2>/dev/null | wc -l); \
			echo "  Generated files: $$output_count"; \
			echo ""; \
		fi; \
	done

# Test that all examples can be built
.PHONY: test
test: clean all
	@echo "=== Testing all examples ==="
	@for dir in $(EXAMPLE_DIRS); do \
		if [ -d $$dir ]; then \
			echo "Testing $$dir..."; \
			output_count=$$(find $$dir -name "output-*" 2>/dev/null | wc -l); \
			if [ "$$output_count" -gt 0 ]; then \
				echo "  ✓ $$output_count files generated"; \
			else \
				echo "  ✗ No output files found"; \
				exit 1; \
			fi; \
		fi; \
	done
	@echo "All tests passed!"

# Help target
.PHONY: help
help:
	@echo "Bicep-Docs Examples Build System"
	@echo "================================="
	@echo ""
	@echo "Main targets:"
	@echo "  all              - Generate all examples in all directories"
	@echo "  clean            - Remove all generated files"
	@echo "  test             - Clean, build all, and verify output"
	@echo ""
	@echo "Format-specific targets (across all directories):"
	@echo "  markdown (md)    - Generate all Markdown examples"
	@echo "  asciidoc (adoc)  - Generate all AsciiDoc examples"
	@echo "  yaml             - Generate all YAML examples"
	@echo "  json             - Generate all JSON examples"
	@echo ""
	@echo "Directory-specific targets:"
	@echo "  single-file      - Build all examples in single-file/"
	@echo "  single-file-clean - Clean examples in single-file/"
	@echo ""
	@echo "Format + directory targets:"
	@echo "  single-file-markdown, single-file-asciidoc, etc."
	@echo ""
	@echo "Utility targets:"
	@echo "  list             - List all available example directories"
	@echo "  status           - Show status of all example directories"
	@echo "  help             - Show this help message"
	@echo ""
	@echo "Example directories: $(EXAMPLE_DIRS)"
