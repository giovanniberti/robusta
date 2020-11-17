use robusta_jni::bridge;

#[bridge]
mod jni {
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::convert::{JValueWrapper, TryFromJavaValue};
    use std::convert::TryInto;
    use robusta_jni::jni::objects::JObject;

    #[package()]
    pub struct User;

    impl<'env> TryFromJavaValue<'env> for User {
        type Source = JObject<'env>;

        fn try_from(_s: Self::Source, _env: &JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self> {
            Ok(User {})
        }
    }

    impl User {
        pub extern "jni" fn userCountStatus(env: &JNIEnv) -> String {
            let users_count: i32 = JValueWrapper::from(env.get_static_field("User", "TOTAL_USERS_COUNT", "I").unwrap()).try_into().unwrap();
            users_count.to_string()
        }

        pub extern "jni" fn hashedPassword(self, env: &JNIEnv, seed: i32) -> String {
            let users_count: i32 = JValueWrapper::from(env.get_static_field("User", "TOTAL_USERS_COUNT", "I").unwrap()).try_into().unwrap();
            users_count.to_string() + "_pass"
        }
    }
}