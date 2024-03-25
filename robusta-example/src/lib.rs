use robusta_jni::bridge;

#[bridge]
mod jni {
    use robusta_jni::convert::{
        Field, IntoJavaValue, Signature, TryFromJavaValue, TryIntoJavaValue,
    };
    use robusta_jni::jni::errors::Error as JniError;
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::convert::Local;
    use robusta_jni::jni::JNIEnv;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(com.example.robusta)]
    pub struct HelloWorld<'env: 'borrow, 'borrow> {
        #[instance]
        raw: Local<'env, 'borrow>,
        #[field]
        foo: Field<'env, 'borrow, String>,
    }

    impl<'env: 'borrow, 'borrow> HelloWorld<'env, 'borrow> {
        #[constructor]
        pub extern "java" fn new(env: &'borrow JNIEnv<'env>) -> JniResult<Self> {}

        pub extern "jni" fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
            input1.push(input2);
            input1.iter().map(ToString::to_string).collect()
        }

        pub extern "jni" fn nativeFun(self, env: &JNIEnv, static_call: bool) -> JniResult<i32> {
            if static_call {
                Ok(HelloWorld::staticJavaAdd(env, 1, 2))
            } else {
                let a = self.javaAdd(env, 0, 0)?;
                Ok(a + self.javaAdd(env, 1, 2)?)
            }
        }

        #[call_type(safe(
            exception_class = "java.lang.IllegalArgumentException",
            message = "something bad happened"
        ))]
        pub extern "jni" fn catchMe(self, _env: &JNIEnv) -> JniResult<i32> {
            Err(JniError::NullPtr("catch me if you can"))
        }

        pub extern "java" fn javaAdd(&self, _env: &JNIEnv, i: i32, u: i32) -> JniResult<i32> {}

        #[call_type(unchecked)]
        pub extern "java" fn staticJavaAdd(env: &JNIEnv, i: i32, u: i32) -> i32 {}

        pub extern "jni" fn setStringHelloWorld(mut self) -> JniResult<()> {
            println!("[rust]: self.foo: \"{}\"", self.foo.get()?);
            self.foo.set("hello world".into())?;
            Ok(())
        }
    }
}
