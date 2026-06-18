PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin

.PHONY: all build release install uninstall clean

all: build

build:
	cargo build

release:
	cargo build --release

install: release
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 target/release/lswt $(DESTDIR)$(BINDIR)/lswt

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/lswt

clean:
	cargo clean
