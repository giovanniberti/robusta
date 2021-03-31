use robusta_jni::bridge;

#[bridge]
pub mod jni {
    use std::convert::TryInto;

    use robusta_jni::convert::{IntoJavaValue, JValueWrapper, Signature, TryFromJavaValue};
    use robusta_jni::jni;
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::objects::JObject;

    #[package()]
    pub struct User<'env: 'borrow, 'borrow> {
        raw: AutoLocal<'env, 'borrow>
    }

    impl<'e: 'b, 'b> Signature for User<'e, 'b> {
        const SIG_TYPE: &'static str = "LUser;";
    }

    impl<'e: 'b, 'b> TryFromJavaValue<'e, 'b> for User<'e, 'b> {
        type Source = JObject<'e>;

        fn try_from(_s: Self::Source, _env: &'b JNIEnv<'e>) -> jni::errors::Result<Self> {
            Ok(User { raw: AutoLocal::new(_env,_s) })
        }
    }

    impl<'e: 'b, 'b> IntoJavaValue<'e> for User<'e, 'b> {
        type Target = JObject<'e>;

        fn into(self, env: &JNIEnv<'e>) -> Self::Target {
            IntoJavaValue::into(&self, env)
        }
    }

    impl<'e: 'b, 'b> Signature for &User<'e, 'b> {
        const SIG_TYPE: &'static str = <User as Signature>::SIG_TYPE;
    }

    impl<'e: 'b, 'b> IntoJavaValue<'e> for &User<'e, 'b> {
        type Target = JObject<'e>;

        fn into(self, _env: &JNIEnv<'e>) -> Self::Target {
            self.raw.as_obj()
        }
    }

    impl<'env: 'borrow, 'borrow> User<'env, 'borrow> {
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

        pub extern "jni" fn hashedPassword(self, env: &JNIEnv, _seed: i32) -> String {
            let user_pw: String = self.getPassword(env).unwrap();
            user_pw + "_pass"
        }

        pub extern "java" fn getPassword(&self, env: &JNIEnv) -> ::robusta_jni::jni::errors::Result<String> {}

        pub extern "java" fn getTotalUsersCount(env: &JNIEnv) -> ::robusta_jni::jni::errors::Result<i32> {}
    }
}
