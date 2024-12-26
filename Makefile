APP_NAME = rustmarks
DESTDIR = /usr/local/bin

build:
	bash scripts/build.sh

build-deb: build
	bash scripts/build-deb.sh

install: build
	sudo cp target/release/$(APP_NAME) $(DESTDIR)/$(APP_NAME)
