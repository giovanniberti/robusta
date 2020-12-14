use robusta_jni::bridge;

#[bridge]
mod jni {
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::convert::{JValueWrapper, TryFromJavaValue, JNIEnvLink, IntoJavaValue};
    use std::convert::TryInto;
    use robusta_jni::jni::objects::JObject;

    #[package()]
    pub struct User<'e> {
        raw: JObject<'e>,
        env: JNIEnv<'e>
    }

    impl<'env, 'r> TryFromJavaValue<'env> for User<'env> {
        type Source = JObject<'env>;

        fn try_from(s: Self::Source, env: &JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self> {
            Ok(User {
                raw: s,
                env: env.clone()
            })
        }
    }

    impl<'e, 'r> JNIEnvLink<'e> for User<'e> {
        fn get_env(&self) -> &JNIEnv<'e> {
            &self.env
        }
    }

    impl<'e, 'r> IntoJavaValue<'e> for User<'e> {
        type Target = JObject<'e>;
        const SIG_TYPE: &'static str = "LUser;";

        fn into(self, _env: &JNIEnv<'e>) -> Self::Target {
            self.raw
        }
    }

    impl<'env, 'r> User<'env> {
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
            let user_pw: String = self.getPassword().unwrap();
            user_pw + "_pass"
        }

        pub extern "java" fn getPassword(self) -> ::robusta_jni::jni::errors::Result<String> {}

        pub extern "java" fn getTotalUsersCount(env: &JNIEnv) -> ::robusta_jni::jni::errors::Result<i32> {}
    }
}
