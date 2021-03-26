use robusta_jni::bridge;

#[bridge]
pub mod jni {
    use std::convert::TryInto;

    use robusta_jni::convert::{IntoJavaValue, JavaValue, JNIEnvLink, JValueWrapper, Signature, TryFromJavaValue};
    use robusta_jni::jni;
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::jni::objects::{JString, JValue};
    use robusta_jni::jni::objects::JObject;

    #[package()]
    pub struct User<'e> {
        raw: JObject<'e>
    }

    impl<'e> Signature for User<'e> {
        const SIG_TYPE: &'static str = "LUser;";
    }

    impl<'e> TryFromJavaValue<'e> for User<'e> {
        type Source = JObject<'e>;

        fn try_from(_s: Self::Source, _env: &JNIEnv<'e>) -> jni::errors::Result<Self> {
            Ok(User { raw: _s })
        }
    }

    impl<'e> IntoJavaValue<'e> for User<'e> {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            IntoJavaValue::into(&self, env)
        }
    }

    impl<'e> IntoJavaValue<'e> for &User<'e> {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            self.raw
        }
    }

    impl<'env> User<'env> {
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
