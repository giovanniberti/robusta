use robusta::bridge;


// Example magic JNI Rust integration (under the hood: `jni` crate)
#[bridge]
mod jni {
    use std::fmt::Display;

    #[package(com.giova.awesomepackage)]
    struct HelloWorld {
        #[static_field]
        pub foo: String,
        pub bar: u32,
        jelly: f64
    }

    impl HelloWorld {
        fn quux(&self, input: String) -> u32 {
            match input.as_str() {
                "" => 0,
                _ => self.bar
            }
        }
    }

    struct Bar {}

    struct Bar2 {}
    impl Bar2 {}

    #[package(com.giova.awesomepackage)]
    enum FooBar {}

    impl Foo {}

    struct Bar3 {}
    impl Bar3 {}
}

fn main() {
    println!("Hello, world!");
}
