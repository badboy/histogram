all: cpp
run: cpp
	./cpp/histo_test

cpp: cpp/histo_test

cpp/histo_test: cpp/histo_test.cpp libhistogram.dylib cpp/histogram.h
	g++ $(CFLAGS) -o $@ cpp/histo_test.cpp libhistogram.dylib

cpp/histogram.h: src/lib.rs src/ffi.rs
	cbindgen -l c++ -o $@ .

libhistogram.dylib: src/lib.rs src/ffi.rs
	cargo build
	cp target/debug/libhistogram.dylib $@
