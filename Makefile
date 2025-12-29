MODE ?= release

BUILT_BIN := target/${MODE}/pwease

.PHONY: all
all: ${BUILT_BIN}

PREFIX ?= /usr/local

.PHONY: install
install: all
	install --group=0 --owner=0 --mode=4755 -t ${PREFIX}/bin ${BUILT_BIN}

${BUILT_BIN}: $(wildcard src/*)
	cargo build --${MODE}
