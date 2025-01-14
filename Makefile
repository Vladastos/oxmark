APP_NAME = rustmarks
DESTDIR = /usr/local/bin


build-deb: build clean-deb
	bash scripts/build-deb.sh

install: build
	sudo cp target/release/$(APP_NAME) $(DESTDIR)/$(APP_NAME)

install-deb: build-deb
	sudo apt install --reinstall -y --allow-downgrades ./target/deb/rustmarks*.deb

build: fmt
	bash scripts/build.sh

fmt:
	cargo fmt

clean-deb:
	rm -rf ./target/deb
