use robusta_jni::bridge;

#[bridge]
pub mod jni {
    use std::convert::TryInto;

    use robusta_jni::convert::{
        IntoJavaValue, JValueWrapper, Signature, TryFromJavaValue, TryIntoJavaValue,
    };
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::JNIEnv;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package()]
    pub struct User<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
        password: String,
    }

    impl<'env: 'borrow, 'borrow> User<'env, 'borrow> {
        pub extern "jni" fn initNative() {
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                std::env::set_var("RUST_LOG", "info");
                "info".to_string()
            });
            println!(
                "Initialized env logger with level: {}",
                std::env::var("RUST_LOG").unwrap()
            );
            env_logger::init();
        }

        pub extern "jni" fn userCountStatus(env: &JNIEnv) -> String {
            let users_count: i32 = JValueWrapper::from(
                env.get_static_field("User", "TOTAL_USERS_COUNT", "I")
                    .unwrap(),
            )
            .try_into()
            .unwrap();
            users_count.to_string()
        }

        pub extern "jni" fn hashedPassword(self, _env: &JNIEnv, _seed: i32) -> String {
            let user_pw: String = self.password;
            user_pw + "_pass"
        }

        pub extern "jni" fn getInt(self, v: i32) -> i32 {
            v
        }

        pub extern "jni" fn getBool(self, v: bool) -> bool {
            v
        }

        pub extern "jni" fn getChar(self, v: char) -> char {
            v
        }

        pub extern "jni" fn getByte(self, v: i8) -> i8 {
            v
        }

        pub extern "jni" fn getFloat(self, v: f32) -> f32 {
            v
        }

        pub extern "jni" fn getDouble(self, v: f64) -> f64 {
            v
        }

        pub extern "jni" fn getLong(self, v: i64) -> i64 {
            v
        }

        pub extern "jni" fn getShort(self, v: i16) -> i16 {
            v
        }

        pub extern "jni" fn getString(self, v: String) -> String {
            v
        }

        pub extern "jni" fn getIntArray(self, v: Vec<i32>) -> Vec<i32> {
            v
        }

        pub extern "jni" fn getStringArray(self, v: Vec<String>) -> Vec<String> {
            v
        }

        pub extern "jni" fn getByteArray(self, v: Box<[u8]>) -> Box<[u8]> {
            v
        }

        pub extern "jni" fn intToString(self, v: i32) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn boolToString(self, v: bool) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn charToString(self, v: char) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn byteToString(self, v: i8) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn floatToString(self, v: f32) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn doubleToString(self, v: f64) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn longToString(self, v: i64) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn shortToString(self, v: i16) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn intArrayToString(self, v: Vec<i32>) -> String {
            format!("{:?}", v)
        }

        pub extern "jni" fn stringArrayToString(self, v: Vec<String>) -> String {
            format!("{:?}", v)
        }

        pub extern "jni" fn byteArrayToString(self, v: Box<[u8]>) -> String {
            format!("{:?}", v)
        }

        pub extern "java" fn getPassword(
            &self,
            env: &JNIEnv,
        ) -> ::robusta_jni::jni::errors::Result<String> {
        }

        pub extern "java" fn getTotalUsersCount(
            env: &JNIEnv,
        ) -> ::robusta_jni::jni::errors::Result<i32> {
        }

        pub extern "java" fn multipleParameters(
            &self,
            env: &JNIEnv,
            v: i32,
            s: String,
        ) -> ::robusta_jni::jni::errors::Result<String> {
        }

        #[constructor]
        pub extern "java" fn new(
            env: &'borrow JNIEnv<'env>,
            username: String,
            password: String,
        ) -> JniResult<Self> {
        }
    }
}
