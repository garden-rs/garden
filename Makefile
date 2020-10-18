# Installation prefix
prefix ?= $(HOME)/.cargo

# External commands and flags.
CARGO ?= cargo
CARGO_FLAGS = --all-targets
ifdef release
    CARGO_FLAGS += --release
endif


# The default "all" target builds the project and runs all tests.
.PHONY: all
all:: build test integration


# make {bench,build,test}
.PHONY: bench build test
bench build test::
	$(CARGO) $@ $(CARGO_FLAGS) $(flags)


# make {clean,doc}
.PHONY: clean doc
clean doc::
	$(CARGO) $@ $(flags)


# Installation
# make DESDIR=/tmp/stage prefix=/usr/local install
.PHONY: install
install::
	$(CARGO) install --path . --root '$(DESTDIR)$(prefix)'


# Integration tests
.PHONY: integration
integration::
	$(CARGO) test --features integration $(CARGO_FLAGS) $(flags)


