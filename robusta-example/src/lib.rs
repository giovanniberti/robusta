use robusta::bridge;


#[bridge]
mod jni {
    #[package(com.giova.awesomepackage)]
    struct HelloWorld {
        //#[static_field]
        pub foo: String,
        pub bar: u32,
        jelly: f64
    }

    impl HelloWorld {
        fn quux(input: String, input2: String) -> String {
            String::new()
        }
    }

    #[package(foo)]
    struct Bar2 {}
    impl Bar2 {}

    //#[package(com.giova.awesomepackage)]
    enum FooBar {}

    //impl Foo {}
}
