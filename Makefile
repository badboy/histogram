all: cpp
.PHONY: all

run: cpp
	./cpp/histo_test
.PHONY: run

cpp: cpp/histo_test
.PHONY: cpp

cpp/histo_test: cpp/histo_test.cpp libhistogram.dylib cpp/histogram.h
	g++ $(CFLAGS) -o $@ cpp/histo_test.cpp libhistogram.dylib

cpp/histogram.h: src/lib.rs src/ffi.rs
	cbindgen -l c++ -o $@ .

libhistogram.dylib: src/lib.rs src/ffi.rs
	cargo build --release
	cp target/release/libhistogram.dylib $@

readme: README.md
.PHONY: readme

README.md: README.tpl src/lib.rs
	cargo readme > $@
