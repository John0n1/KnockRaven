# Makefile for Knockraven
#
# This Makefile provides convenience targets for building, testing and
# packaging the Knockraven project.  It delegates compilation to Cargo but
# also exposes a simple install target and a dist target for creating an
# archive suitable for distribution on Linux.

CARGO ?= cargo

.PHONY: all build clean run test install dist
 .PHONY: deb

# Default target: build release binary
all: build

build:
	$(CARGO) build --release

clean:
	$(CARGO) clean

run: build
	./target/release/knockraven $(ARGS)

test:
	$(CARGO) test

# Install the release binary into ~/.cargo/bin
install: build
	$(CARGO) install --path .

# Create a tarball containing the binary and documentation
dist: build
	mkdir -p dist
	cp target/release/knockraven dist/
	cp README.md dist/
	cp LICENSE dist/
	cp -r docs dist/
	tar czf knockraven.tar.gz -C dist .
	@echo "Distribution created: knockraven.tar.gz"

# Build a Debian package using the metadata in the debian/ directory.
# This target requires dpkg-dev and debhelper to be installed.  The
# resulting .deb package will be placed in the parent directory.
deb: clean
	dpkg-buildpackage -us -uc -b