use robusta::bridge;


#[bridge]
mod jni {
    #[package(com.example.robusta)]
    struct HelloWorld;

    impl HelloWorld {
        fn special(input1: String, input2: String) -> String {
            format!("{} + {} = :D!", input1, input2)
        }
    }
}
