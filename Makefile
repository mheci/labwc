.PHONY: all build release test clean install check fmt clippy docs arch-pkg fedora-pkg

all: build

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

check:
	cargo check --workspace

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

docs:
	cargo doc --no-deps --document-private-items
	@echo "Docs at target/doc/labwc_rs/index.html"

clean:
	cargo clean

install: release
	install -Dm755 target/release/labwc-rs $(DESTDIR)/usr/bin/labwc-rs
	install -Dm644 LICENSE $(DESTDIR)/usr/share/licenses/labwc-rs/LICENSE

arch-pkg: release
	cd packaging/arch && makepkg -sf

fedora-pkg: release
	mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	cp packaging/fedora/labwc-rs.spec ~/rpmbuild/SPECS/
	rpmbuild -bb ~/rpmbuild/SPECS/labwc-rs.spec
