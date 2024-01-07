build:
	cargo build

# https://stackoverflow.com/a/12099167/13500870
lib_path = 
ifeq ($(OS),Windows_NT)
	lib_path = PATH=${PATH};${JAVA_HOME}\bin;${JAVA_HOME}\bin\server
else
	UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Darwin)
        lib_path = DYLD_FALLBACK_LIBRARY_PATH=${DYLD_FALLBACK_LIBRARY_PATH}:${JAVA_HOME}/lib:${JAVA_HOME}/lib/server
	else
		lib_path = LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:${JAVA_HOME}/lib:${JAVA_HOME}/lib/server
	endif
endif

test:
	$(lib_path) cargo test  -- --test-threads=1

.PHONY: build test all

all: build test

.DEFAULT_GOAL := all
