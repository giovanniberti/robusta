use robusta::bridge;

#[bridge]
mod jni {
    #[package(com.example.robusta)]
    struct HelloWorld;

    impl HelloWorld {
        fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
            input1.push(input2);
            input1.iter().map(ToString::to_string).collect()
        }
    }
}
