# Installation prefix
prefix ?= $(HOME)/.cargo

# External commands and flags.
CARGO ?= cargo
CARGO_FLAGS =
MDBOOK ?= mdbook

CARGO_PACKAGE = garden-tools
ifdef release
    CARGO_FLAGS += --release
endif

ifdef V
    VERBOSE = 1
endif

ifndef VERBOSE
.SILENT:
else
    CARGO_FLAGS += -v
endif

# The default "all" target builds the project and runs all tests.
.PHONY: all
all:: build


.PHONY: bench build test
bench build test::
	$(CARGO) $@ --all-targets $(CARGO_FLAGS) $(flags)

.PHONY: clean
clean::
	$(CARGO) clean $(flags)

.PHONY: doc
doc::
	$(CARGO) doc --no-deps --package $(CARGO_PACKAGE)
	cd doc && $(MDBOOK) build --dest-dir book.new
	rsync -r --delete --exclude=.git doc/book.new/ doc/book/
	rm -rf doc/book.new


# Installation
# make DESDIR=/tmp/stage prefix=/usr/local install
.PHONY: install
install::
	$(CARGO) install --path . --root '$(DESTDIR)$(prefix)'


.PHONY: install-doc
install-doc:: doc
	mkdir -p $(DESTDIR)$(prefix)/share/doc/garden
	rsync -r doc/book/ $(DESTDIR)$(prefix)/share/doc/garden/


# Integration tests
.PHONY: test-integration
test-integration::
	$(CARGO) test --features integration $(CARGO_FLAGS) $(flags)


# Test coverage
.PHONY: coverage
coverage::
	cargo kcov --verbose


# Tests and checks
.PHONY: check
check:: check-clippy
check:: check-format


# Check for common mistakes and code improvements using clippy.
.PHONY: check-clippy
check-clippy::
	cargo clippy --all -- -D warnings


# Apply clippy suggestions.
.PHONY: clippy-fix
clippy-fix:
	cargo clippy --fix $(flags)


# Code formatting
.PHONY: check-format
check-format::
	$(CARGO) fmt -- --check \
	|| echo "# Changes detected.  Run 'make format' to apply changes."

.PHONY: format
format::
	$(CARGO) fmt
