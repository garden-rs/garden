# Installation prefix
prefix ?= $(HOME)/.cargo

# External commands and flags.
CARGO ?= cargo
CARGO_FLAGS =
CARGO_PACKAGE = garden-tools

ifndef debug
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


# Installation
# make DESDIR=/tmp/stage prefix=/usr/local install
.PHONY: install
install::
	$(CARGO) install --path . --root '$(DESTDIR)$(prefix)'


# Integration tests
.PHONY: test-integration
test-integration::
	$(CARGO) test --features integration $(CARGO_FLAGS) $(flags)


.PHONY: coverage
coverage::
	cargo kcov --verbose


.PHONY:check
check::
	cargo clippy --all -- -D warnings


# Code formatting
.PHONY: check-format
check-format::
	$(CARGO) fmt -- --check \
	|| echo "# Changes detected.  Run 'make format' to apply changes."

.PHONY: format
format::
	$(CARGO) fmt
