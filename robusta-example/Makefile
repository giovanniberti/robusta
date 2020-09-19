java_run: lib
	javac com/example/robusta/HelloWorld.java && RUST_BACKTRACE=full java -Djava.library.path=../target/debug com.example.robusta.HelloWorld

.PHONY: lib

lib:
	cargo build
