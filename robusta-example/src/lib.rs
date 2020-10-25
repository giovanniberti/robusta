use robusta_jni::bridge;

#[bridge]
mod jni {
    use jni::objects::JObject;
    use robusta_jni::convert::JNIEnvLink;
    use robusta_jni::jni::JNIEnv;
    use std::marker::PhantomData;

    #[package(com.example.robusta)]
    pub struct HelloWorld<'e, 'a> {
        env: JNIEnv<'e>,
        marker: PhantomData<&'a ()>,
    }

    impl<'e, 'a> ::robusta_jni::convert::IntoJavaValue<'e> for HelloWorld<'e, 'a> {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            env.new_object("com/example/robusta/HelloWorld", "()V", &[])
                .unwrap()
        }
    }

    impl<'e, 'a> ::robusta_jni::convert::IntoJavaValue<'e> for &HelloWorld<'e, 'a> {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            env.new_object("com/example/robusta/HelloWorld", "()V", &[])
                .unwrap()
        }
    }

    impl<'e, 'a> ::robusta_jni::convert::FromJavaValue<'e> for HelloWorld<'e, 'a> {
        type Source = JObject<'e>;

        fn from(s: Self::Source, env: &JNIEnv<'e>) -> Self {
            HelloWorld {
                env: env.clone(),
                marker: PhantomData,
            }
        }
    }

    impl<'e, 'a> JNIEnvLink<'e> for HelloWorld<'e, 'a> {
        fn get_env(&self) -> &JNIEnv<'e> {
            &self.env
        }
    }

    impl<'env, 'a> HelloWorld<'env, 'a> {
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
                self.javaAdd(1, 2).unwrap()
            }
        }

        #[call_type(safe)]
        pub extern "java" fn javaAdd(
            &self,
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
