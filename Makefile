build:
	cargo build

lib_path = "${LD_LIBRARY_PATH}:${JAVA_HOME}/lib:${JAVA_HOME}/lib/server"

test:
	LD_LIBRARY_PATH=$(lib_path) cargo test  -- --test-threads=1

.PHONY: build test all

all: build test

.DEFAULT_GOAL := all
