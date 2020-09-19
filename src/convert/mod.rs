use std::str::FromStr;

use jni::JNIEnv;
use jni::objects::{JObject, JString};
use jni::signature::JavaType;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use paste::paste;

pub use safe::*;
pub use unchecked::*;

mod safe;
mod unchecked;

/// A trait for types that are ffi-safe to use with JNI. It is implemented for primitives, [`jni::objects::JObject`] and [`jni::sys::jobject`].
/// User that wants automatic conversion should instead implement [`FromJavaValue`] and [`IntoJavaValue`]
pub trait JavaValue<'env> {
    fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env>;

    fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self;
}

macro_rules! jvalue_types {
    ($type:ty: $boxed:ident ($sig:ident) [$unbox_method:ident]) => {
        impl<'env> JavaValue<'env> for $type {
            fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env> {
                env.call_static_method_unchecked(concat!("java/lang/", stringify!($boxed)),
                    (concat!("java/lang/", stringify!($boxed)), "valueOf", concat!(stringify!(($sig)), "Ljava/lang/", stringify!($boxed), ";")),
                    JavaType::from_str(concat!("java/lang/", stringify!($boxed))).unwrap(),
                    &[Into::into(self)]).unwrap().l().unwrap()
            }

            fn unbox(s: JObject<'env>, env:&JNIEnv<'env>) -> Self {
                paste!(Into::into(env.call_method_unchecked(s, (concat!("java/lang/", stringify!($boxed)), stringify!($unbox_method), concat!("()", stringify!($sig))), JavaType::from_str(stringify!($sig)).unwrap(), &[])
                    .unwrap().[<$sig:lower>]()
                    .unwrap()))
            }
        }
    };

    ($type:ty: $boxed:ident ($sig:ident) [$unbox_method:ident], $($rest:ty: $rest_boxed:ident ($rest_sig:ident) [$unbox_method_rest:ident]),+) => {
        jvalue_types!($type: $boxed ($sig) [$unbox_method]);

        jvalue_types!($($rest: $rest_boxed ($rest_sig) [$unbox_method_rest]),+);
    }
}

jvalue_types! {
    jboolean: Boolean (Z) [booleanValue],
    jbyte: Byte (B) [byteValue],
    jchar: Character (C) [charValue],
    jdouble: Double (D) [doubleValue],
    jfloat: Float (F) [floatValue],
    jint: Integer (I) [intValue],
    jlong: Long (J) [longValue],
    jshort: Short (S) [shortValue]
}

impl<'env> JavaValue<'env> for JObject<'env> {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        self
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        s
    }
}

impl<'env> JavaValue<'env> for jobject {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        From::from(self)
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        s.into_inner()
    }
}

impl<'env> JavaValue<'env> for JString<'env> {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        Into::into(self)
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        From::from(s)
    }
}
