use robusta_jni::convert::{ArrSignature, FromJavaValue, IntoJavaValue, Signature, TryFromJavaValue, TryIntoJavaValue};
use robusta_jni::jni::JNIEnv;

#[derive(Signature, ArrSignature)]
#[array(String)]
pub struct StringArr {
    value: Box<[String]>,
}

impl From<Box<[String]>> for StringArr {
    fn from(value: Box<[String]>) -> Self {
        Self { value }
    }
}

impl From<StringArr> for Box<[String]> {
    fn from(v: StringArr) -> Self {
        v.value
    }
}

impl<'env> TryIntoJavaValue<'env> for StringArr {
    type Target = <Box<[String]> as TryIntoJavaValue<'env>>::Target;

    fn try_into(self, env: &JNIEnv<'env>) -> robusta_jni::jni::errors::Result<Self::Target> {
        TryIntoJavaValue::try_into(<Box<[String]> as From<Self>>::from(self), env)
    }
}

impl<'env> IntoJavaValue<'env> for StringArr {
    type Target = <Box<[String]> as TryIntoJavaValue<'env>>::Target;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        IntoJavaValue::into(<Box<[String]> as From<Self>>::from(self), env)
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for StringArr {
    type Source = <Box<[String]> as TryFromJavaValue<'env, 'borrow>>::Source;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> robusta_jni::jni::errors::Result<Self> {
        <Box<[String]> as TryFromJavaValue>::try_from(s, env).map(|res|
            <Self as From<Box<[String]>>>::from(res)
        )
    }
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for StringArr {
    type Source = <Box<[String]> as FromJavaValue<'env, 'borrow>>::Source;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        <Self as From<Box<[String]>>>::from(<Box<[String]> as FromJavaValue>::from(s, env))
    }
}

#[robusta_jni::bridge]
pub mod jni {
    use std::convert::TryInto;

    use robusta_jni::convert::{Field, JavaValue, JValueWrapper};
    use robusta_jni::convert::{IntoJavaValue, FromJavaValue, Signature, ArrSignature, TryFromJavaValue, TryIntoJavaValue};
    use robusta_jni::convert::Local;
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::JNIEnv;
    use crate::StringArr;

    #[derive(Signature, ArrSignature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue, FromJavaValue)]
    #[package()]
    pub struct User<'env: 'borrow, 'borrow> {
        #[instance]
        raw: Local<'env, 'borrow>,
        #[field]
        pub username: Field<'env, 'borrow, String>,
        pub password: String,
        #[field]
        pub bytes: Field<'env, 'borrow, Box<[i8]>>,
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

        #[call_type(unchecked)]
        pub extern "jni" fn getIntUnchecked(v: i32) -> i32 {
            v
        }

        pub extern "jni" fn getBool(self, v: bool) -> bool {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getBoolUnchecked(v: bool) -> bool {
            v
        }

        pub extern "jni" fn getChar(self, v: char) -> char {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getCharUnchecked(v: char) -> char {
            v
        }

        pub extern "jni" fn getByte(self, v: i8) -> i8 {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getByteUnchecked(v: i8) -> i8 {
            v
        }

        pub extern "jni" fn getFloat(self, v: f32) -> f32 {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getFloatUnchecked(v: f32) -> f32 {
            v
        }

        pub extern "jni" fn getDouble(self, v: f64) -> f64 {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getDoubleUnchecked(v: f64) -> f64 {
            v
        }

        pub extern "jni" fn getLong(self, v: i64) -> i64 {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getLongUnchecked(v: i64) -> i64 {
            v
        }

        pub extern "jni" fn getShort(self, v: i16) -> i16 {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getShortUnchecked(v: i16) -> i16 {
            v
        }

        pub extern "jni" fn getString(self, v: String) -> String {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getStringUnchecked(v: String) -> String {
            v
        }

        pub extern "jni" fn getIntArray(self, v: Vec<i32>) -> Vec<i32> {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getIntArrayUnchecked(v: Vec<i32>) -> Vec<i32> {
            v
        }

        pub extern "jni" fn getStringArray(self, v: Vec<String>) -> Vec<String> {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getStringArrayUnchecked(v: Vec<String>) -> Vec<String> {
            v
        }

        pub extern "jni" fn getByteArray(self, v: Box<[i8]>) -> Box<[i8]> {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getByteArrayUnchecked(v: Box<[i8]>) -> Box<[i8]> {
            v
        }

        pub extern "jni" fn getBoolArray(self, v: Box<[bool]>) -> Box<[bool]> {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getBoolArrayUnchecked(v: Box<[bool]>) -> Box<[bool]> {
            v
        }

        pub extern "jni" fn getJStringArr(self, v: Box<[robusta_jni::jni::objects::JString<'env>]>) -> Box<[robusta_jni::jni::objects::JString<'env>]> {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getJStringArrUnchecked(v: Box<[robusta_jni::jni::objects::JString<'env>]>) -> Box<[robusta_jni::jni::objects::JString<'env>]> {
            v
        }

        pub extern "jni" fn getStringArr(self, v: Box<[String]>) -> Box<[String]> {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getStringArrUnchecked(v: Box<[String]>) -> Box<[String]> {
            v
        }

        pub extern "jni" fn getOptionStringArr(self, v: Box<[Option<String>]>) -> Box<[Option<String>]> {
            v.iter().map(|item|
                match item {
                    Some(str) => if str == "null" { None } else { Some(str.clone()) },
                    None => Some("null".to_string()),
                }
            ).collect()
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getOptionStringArrUnchecked(v: Box<[Option<String>]>) -> Box<[Option<String>]> {
            v.iter().map(|item|
                match item {
                    Some(str) => if str == "null" { None } else { Some(str.clone()) },
                    None => Some("null".to_string()),
                }
            ).collect()
        }

        pub extern "jni" fn getOptionString(self, v: Option<String>) -> Option<String> {
            v
        }

        #[call_type(unchecked)]
        pub extern "jni" fn getOptionStringUnchecked(v: Option<String>) -> Option<String> {
            v
        }

        pub extern "jni" fn intToString(self, v: i32) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn intToStringUnchecked(v: i32) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn boolToString(self, v: bool) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn boolToStringUnchecked(v: bool) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn charToString(self, v: char) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn charToStringUnchecked(v: char) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn byteToString(self, v: i8) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn byteToStringUnchecked(v: i8) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn floatToString(self, v: f32) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn floatToStringUnchecked(v: f32) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn doubleToString(self, v: f64) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn doubleToStringUnchecked(v: f64) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn longToString(self, v: i64) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn longToStringUnchecked(v: i64) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn shortToString(self, v: i16) -> String {
            format!("{}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn shortToStringUnchecked(v: i16) -> String {
            format!("{}", v)
        }

        pub extern "jni" fn intArrayToString(self, v: Vec<i32>) -> String {
            format!("{:?}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn intArrayToStringUnchecked(v: Vec<i32>) -> String {
            format!("{:?}", v)
        }

        pub extern "jni" fn stringArrayToString(self, v: Vec<String>) -> String {
            format!("{:?}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn stringArrayToStringUnchecked(v: Vec<String>) -> String {
            format!("{:?}", v)
        }

        pub extern "jni" fn stringArrToString(self, v: Box<[String]>) -> String {
            format!("{:?}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn stringArrToStringUnchecked(v: Box<[String]>) -> String {
            format!("{:?}", v)
        }

        pub extern "jni" fn byteArrayToString(self, v: Box<[i8]>) -> String {
            format!("{:?}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn byteArrayToStringUnchecked(v: Box<[i8]>) -> String {
            format!("{:?}", v)
        }

        pub extern "jni" fn boolArrayToString(self, v: Box<[bool]>) -> String {
            format!("{:?}", v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn boolArrayToStringUnchecked(v: Box<[bool]>) -> String {
            format!("{:?}", v)
        }

        pub extern "java" fn nullableString(
            env: &JNIEnv,
            v: Option<String>,
        ) -> JniResult<Option<String>> {}

        #[call_type(unchecked)]
        pub extern "java" fn nullableStringUnchecked(
            env: &JNIEnv,
            v: Option<String>,
        ) -> Option<String> {}

        pub extern "java" fn nullableDouble(
            env: &JNIEnv,
            v: Option<f64>,
        ) -> JniResult<f64> {}

        #[call_type(unchecked)]
        pub extern "java" fn nullableDoubleUnchecked(
            env: &JNIEnv,
            v: Option<f64>,
        ) -> f64 {}


        pub extern "java" fn getPassword(
            &self,
            env: &JNIEnv,
        ) -> JniResult<String> {}

        #[call_type(unchecked)]
        pub extern "java" fn getPasswordUnchecked(
            &self,
            env: &JNIEnv,
        ) -> String {}

        pub extern "java" fn getTotalUsersCount(
            env: &JNIEnv,
        ) -> JniResult<i32> {}

        #[call_type(unchecked)]
        pub extern "java" fn getTotalUsersCountUnchecked(
            env: &JNIEnv,
        ) -> i32 {}

        pub extern "java" fn multipleParameters(
            &self,
            env: &JNIEnv,
            v: i32,
            s: String,
        ) -> JniResult<String> {}

        #[call_type(unchecked)]
        pub extern "java" fn multipleParametersUnchecked(
            &self,
            env: &JNIEnv,
            v: i32,
            s: String,
        ) -> String {}

        pub extern "java" fn stringArrNullable2D(
            env: &JNIEnv,
            a: Option<StringArr>,
            b: Option<StringArr>,
        ) -> JniResult<Box<[Option<StringArr>]>> {}

        #[call_type(unchecked)]
        pub extern "java" fn stringArrNullable2DUnchecked(
            &self,
            env: &JNIEnv,
            a: Option<StringArr>,
            b: Option<StringArr>,
        ) -> Box<[Option<StringArr>]> {}

        pub extern "java" fn signaturesCheck(
            &self,
            env: &'borrow JNIEnv<'env>,
            int: i32,
            boolean: bool,
            character: char,
            byte: i8,
            float: f32,
            double: f64,
            long: i64,
            short: i16,
            string: String,
            int_array: Vec<i32>,
            string_array: Vec<String>,
            nullable_string_array: Vec<Option<String>>,
            byte_array: Box<[i8]>,
            bool_array: Box<[bool]>,
            jstring_arr: Box<[robusta_jni::jni::objects::JString<'env>]>,
            string_arr: Box<[String]>,
            string_arr_nullable: Box<[Option<String>]>,
            nullable_string: Option<String>,
            byte_array_nullable_2d: Vec<Option<Box<[i8]>>>,
            byte_array_2d: Vec<Box<[i8]>>,
            string_array_nullable_2d: Vec<Option<Box<[String]>>>,
            string_array_2d: Vec<Box<[String]>>,
            string_arr_nullable_2d: Box<[Option<StringArr>]>,
            string_arr_2d: Box<[StringArr]>,
        ) -> JniResult<Vec<String>> {}

        #[call_type(unchecked)]
        pub extern "java" fn signaturesCheckUnchecked(
            env: &'borrow JNIEnv<'env>,
            int: i32,
            boolean: bool,
            character: char,
            byte: i8,
            float: f32,
            double: f64,
            long: i64,
            short: i16,
            string: String,
            int_array: Vec<i32>,
            string_array: Vec<String>,
            nullable_string_array: Vec<Option<String>>,
            byte_array: Box<[i8]>,
            bool_array: Box<[bool]>,
            jstring_arr: Box<[robusta_jni::jni::objects::JString<'env>]>,
            string_arr: Box<[String]>,
            string_arr_nullable: Box<[Option<String>]>,
            nullable_string: Option<String>,
            byte_array_nullable_2d: Vec<Option<Box<[i8]>>>,
            byte_array_2d: Vec<Box<[i8]>>,
            string_array_nullable_2d: Vec<Option<Box<[String]>>>,
            string_array_2d: Vec<Box<[String]>>,
            string_arr_nullable_2d: Box<[Option<StringArr>]>,
            string_arr_2d: Box<[StringArr]>,
        ) -> Vec<String> {}

        pub extern "java" fn selfSignatureCheck(
            &self,
            env: &'borrow JNIEnv<'env>,
            user: Self,
            borrow_user: &Self,
            nullable_borrow_user1: Option<&Self>,
            nullable_borrow_user2: Option<&Self>,
            nullable_user1: Option<Self>,
            nullable_user2: Option<Self>,
            user_array: Vec<Self>,
            nullable_user_array: Vec<Option<Self>>,
            user_array_nullable1: Option<Vec<Self>>,
            user_array_nullable2: Option<Vec<Self>>,
            user_arr: Box<[Self]>,
            nullable_user_arr: Box<[Option<Self>]>,
            user_arr_nullable1: Option<Box<[Self]>>,
            user_arr_nullable2: Option<Box<[Self]>>,
        ) -> JniResult<Vec<String>> {}

        #[call_type(unchecked)]
        pub extern "java" fn selfSignatureCheckUnchecked(
            &self,
            env: &'borrow JNIEnv<'env>,
            user: Self,
            borrow_user: &Self,
            nullable_borrow_user1: Option<&Self>,
            nullable_borrow_user2: Option<&Self>,
            nullable_user1: Option<Self>,
            nullable_user2: Option<Self>,
            user_array: Vec<Self>,
            nullable_user_array: Vec<Option<Self>>,
            user_array_nullable1: Option<Vec<Self>>,
            user_array_nullable2: Option<Vec<Self>>,
            user_arr: Box<[Self]>,
            nullable_user_arr: Box<[Option<Self>]>,
            user_arr_nullable1: Option<Box<[Self]>>,
            user_arr_nullable2: Option<Box<[Self]>>,
        ) -> Vec<String> {}

        #[call_type(unchecked)]
        pub extern "java" fn cloneUser(
            env: &'borrow JNIEnv<'env>,
            user: &Self,
        ) -> Self {}

        #[constructor]
        pub extern "java" fn new(
            env: &'borrow JNIEnv<'env>,
            username: String,
            password: String,
        ) -> JniResult<Self> {}

        #[call_type(unchecked)]
        #[constructor]
        pub extern "java" fn newUnchecked(
            env: &'borrow JNIEnv<'env>,
            username: String,
        ) -> Self {}

        #[call_type(unchecked)]
        pub extern "java" fn toString(
            &self,
            env: &JNIEnv,
        ) -> String {}

        // No type checks here, unbox and autobox use call_method_unchecked, be extra careful.
        // If you replace <f64 as JavaValue>::unbox with <i8 as JavaValue>::unbox, it won't fail, but it's an UB.
        // It's better to define a wrapper struct and impl Signature for it.
        #[output_type("Ljava/lang/Double;")]
        pub extern "java" fn typeOverrideJava(
            &self,
            env: &'borrow JNIEnv<'env>,
            #[input_type("Ljava/lang/Double;")]
            v: robusta_jni::jni::objects::JObject<'env>
        ) -> JniResult<robusta_jni::jni::objects::JObject<'env>> {}

        #[call_type(unchecked)]
        #[output_type("Ljava/lang/Double;")]
        pub extern "java" fn typeOverrideJavaUnchecked(
            env: &'borrow JNIEnv<'env>,
            #[input_type("Ljava/lang/Double;")]
            v: robusta_jni::jni::objects::JObject<'env>
        ) -> robusta_jni::jni::objects::JObject<'env> {}


        // Rust signatures aren't involved here, only Java definition matters
        pub extern "jni" fn typeOverrideJni(
            self, env: &'borrow JNIEnv<'env>,
            v: robusta_jni::jni::objects::JObject<'env>
        ) -> robusta_jni::jni::objects::JObject<'env> {
            Self::typeOverrideJniUnchecked(env, v)
        }

        #[call_type(unchecked)]
        pub extern "jni" fn typeOverrideJniUnchecked(
            env: &'borrow JNIEnv<'env>,
            v: robusta_jni::jni::objects::JObject<'env>
        ) -> robusta_jni::jni::objects::JObject<'env> {
            let val: f64 = JavaValue::unbox(v, env);
            (val * -10f64).autobox(env)
        }
    }
}
