use robusta_jni::bridge;

#[bridge]
pub mod jni {
    use std::convert::TryInto;

    use robusta_jni::convert::{IntoJavaValue, JavaValue, JNIEnvLink, JValueWrapper, TryFromJavaValue};
    use robusta_jni::convert::handle::{Handle, HandleDispatcher, Signature};
    use robusta_jni::jni;
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::jni::objects::{JString, JValue};
    use robusta_jni::jni::objects::JObject;

    #[package()]
    pub struct User;

    impl<'e> Signature for User {
        const SIG_TYPE: &'static str = "LUser;";
    }

    impl<'e> HandleDispatcher<'e> for User {
        type Handle = Handle<'e, User>;
    }

    impl<'e> TryFromJavaValue<'e> for User {
        type Source = JObject<'e>;

        fn try_from(_s: Self::Source, _env: &JNIEnv<'e>) -> jni::errors::Result<Self> {
            Ok(User)
        }
    }

    impl<'e> IntoJavaValue<'e> for User {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            IntoJavaValue::into(&self, env)
        }
    }

    impl<'e> IntoJavaValue<'e> for &User {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            // TODO: Document that inside `IntoJavaValue` you cannot call Java methods (otherwise infinite recursion happens)
            let user_string = env.new_string("user").expect("Can't create username string");
            // should be env.new_string(self.getPassword())
            let pw_string = env.new_string("pass").expect("Can't create password string");
            env.new_object("User", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(JObject::from(user_string)), JValue::Object(JObject::from(pw_string))])
                .unwrap()
        }
    }

    impl User {
        pub extern "jni" fn initNative() {
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                std::env::set_var("RUST_LOG", "info");
                "info".to_string()
            });
            println!("Initialized env logger with level: {}", std::env::var("RUST_LOG").unwrap());
            env_logger::init();
        }

        pub extern "jni" fn userCountStatus(env: &JNIEnv) -> String {
            let users_count: i32 = JValueWrapper::from(env.get_static_field("User", "TOTAL_USERS_COUNT", "I").unwrap()).try_into().unwrap();
            users_count.to_string()
        }

        pub extern "jni" fn hashedPassword(self, env: &JNIEnv, seed: i32) -> String {
            let user_pw: String = self.getPassword(env).unwrap();
            user_pw + "_pass"
        }

        pub extern "java" fn getPassword(&self, env: &JNIEnv) -> ::robusta_jni::jni::errors::Result<String> {}

        pub extern "java" fn getTotalUsersCount(env: &JNIEnv) -> ::robusta_jni::jni::errors::Result<i32> {}
    }
}
