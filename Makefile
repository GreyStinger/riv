all: clean build

build:
	cargo build --release

clean:
	rm -rf target/deps/
	rm -f target/riv target/riv.d

install: all
	mkdir -p ${DESTDIR}${PREFIX}/bin
	cp -f target/release/riv ${DESTDIR}${PREFIX}/bin
	chmod 755 ${DESTDIR}${PREFIX}/bin/riv

uninstall:
	rm -f ${DESTDIR}${PREFIX}/bin/riv

.PHONY: all build clean install uninstall
