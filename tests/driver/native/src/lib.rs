use robusta_jni::bridge;

#[bridge]
pub mod jni {
    use std::convert::TryInto;

    use robusta_jni::convert::{IntoJavaValue, JValueWrapper, Signature, TryFromJavaValue, TryIntoJavaValue};
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::jni::objects::AutoLocal;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package()]
    pub struct User<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
        password: String
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

        pub extern "jni" fn hashedPassword(self, _env: &JNIEnv, _seed: i32) -> String {
            let user_pw: String = self.password;
            user_pw + "_pass"
        }

        pub extern "java" fn getPassword(&self, env: &JNIEnv) -> ::robusta_jni::jni::errors::Result<String> {}

        pub extern "java" fn getTotalUsersCount(env: &JNIEnv) -> ::robusta_jni::jni::errors::Result<i32> {}

        #[constructor]
        pub extern "java" fn new(env: &'borrow JNIEnv<'env>, username: String, password: String) -> ::robusta_jni::jni::errors::Result<Self> {}
    }
}
