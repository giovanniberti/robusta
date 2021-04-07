use robusta_jni::bridge;

#[bridge]
mod jni {
    use robusta_jni::convert::{Signature, IntoJavaValue, FromJavaValue, TryIntoJavaValue, TryFromJavaValue};
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::errors::Result as JniResult;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, FromJavaValue, TryFromJavaValue)]
    #[package(com.example.robusta)]
    pub struct HelloWorld<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>
    }

    impl<'env: 'borrow, 'borrow> HelloWorld<'env, 'borrow> {

        #[constructor]
        pub extern "java" fn new(env: &'borrow JNIEnv<'env>) -> JniResult<Self> {}

        #[call_type(unchecked)]
        pub extern "jni" fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
            input1.push(input2);
            input1.iter().map(ToString::to_string).collect()
        }

        #[call_type(unchecked)]
        pub extern "jni" fn nativeFun(self, _env: &JNIEnv, static_call: bool) -> i32 {
            if static_call {
                HelloWorld::staticJavaAdd(_env, 1, 2)
            } else {
                self.javaAdd(_env, 1, 2).unwrap()
            }
        }

        pub extern "java" fn javaAdd(
            &self,
            _env: &JNIEnv,
            i: i32,
            u: i32,
        ) -> jni::errors::Result<i32> {}

        #[call_type(unchecked)]
        pub extern "java" fn staticJavaAdd(env: &JNIEnv, i: i32, u: i32) -> i32 {}
    }
}
