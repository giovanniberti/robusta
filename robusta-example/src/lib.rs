use robusta_jni::bridge;

#[bridge]
mod jni {
    use jni::objects::JObject;

    use robusta_jni::convert::{Signature, IntoJavaValue, FromJavaValue, TryIntoJavaValue};
    use robusta_jni::jni::JNIEnv;

    #[derive(Signature)]
    #[package(com.example.robusta)]
    pub struct HelloWorld {
        _marker: (),
    }

    impl<'e> IntoJavaValue<'e> for HelloWorld {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            env.new_object("com/example/robusta/HelloWorld", "()V", &[])
                .unwrap()
        }
    }

    impl<'e> IntoJavaValue<'e> for &HelloWorld {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            env.new_object("com/example/robusta/HelloWorld", "()V", &[])
                .unwrap()
        }
    }

    impl<'e: 'b, 'b> FromJavaValue<'e, 'b> for HelloWorld {
        type Source = JObject<'e>;

        fn from(_s: Self::Source, _env: &'b JNIEnv<'e>) -> Self {
            HelloWorld {
                _marker: (),
            }
        }
    }

    impl<'e: 'b, 'b> TryIntoJavaValue<'e> for HelloWorld {
        type Target = JObject<'e>;

        fn try_into(self, env: &JNIEnv<'e>) -> robusta_jni::jni::errors::Result<Self::Target> {
            TryIntoJavaValue::try_into(&self, env)
        }
    }

    impl<'e: 'b, 'b> TryIntoJavaValue<'e> for &HelloWorld {
        type Target = JObject<'e>;

        fn try_into(self, env: &JNIEnv<'e>) -> robusta_jni::jni::errors::Result<Self::Target> {
            Ok(IntoJavaValue::into(self, env))
        }
    }

    impl HelloWorld {
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
