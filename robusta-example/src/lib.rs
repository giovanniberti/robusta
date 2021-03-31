use robusta_jni::bridge;

#[bridge]
mod jni {
    use std::marker::PhantomData;

    use jni::objects::JObject;

    use robusta_jni::convert::{JNIEnvLink, Signature, IntoJavaValue, FromJavaValue};
    use robusta_jni::jni::JNIEnv;

    #[derive(Signature)]
    #[package(com.example.robusta)]
    pub struct HelloWorld {
        marker: (),
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

        fn from(s: Self::Source, env: &'b JNIEnv<'e>) -> Self {
            HelloWorld {
                marker: (),
            }
        }
    }

    impl HelloWorld {
        #[call_type(safe)]
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
        ) -> ::robusta_jni::jni::errors::Result<i32> {}

        #[call_type(unchecked)]
        pub extern "java" fn staticJavaAdd(
            env: &JNIEnv,
            i: i32,
            u: i32,
        ) -> i32 {}
    }
}
