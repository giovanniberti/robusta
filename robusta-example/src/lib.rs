use robusta::bridge;

#[bridge]
mod jni {
    #[package(com.example.robusta)]
    struct HelloWorld;

    impl HelloWorld {
        fn special(mut input1: i32, input2: i32) -> i32 {
            /*input1.push(input2);
            input1.iter().map(ToString::to_string).collect()*/
            input1 + input2
        }
    }
}
