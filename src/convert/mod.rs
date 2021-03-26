//! Conversion facilities.
//! This module provides two trait families: [FromJavaValue]/[IntoJavaValue] (infallible conversions) and [TryFromJavaValue]/[TryIntoJavaValue] (fallible conversions),
//! similar to the ones found in the standard library.
//!
//! The `call_type` attribute controls which of the two conversion families is selected during code generation.
//! `call_type` is a per-function attribute.
//! Specific parameters that can be given to `call_type` can be found in the module documentation relative to the trait family ([safe] module for fallible conversions and [unchecked] module for infallible conversions)
//!
//! **If the `call_type` attribute is omitted, the fallible conversion trait family is chosen.**
//!
//! Example usage:
//! ```
//! use robusta_jni::bridge;
//!
//! #[bridge]
//! mod jni {
//!     #[package(com.example.robusta)]
//!     struct HelloWorld;
//!
//!     impl HelloWorld {
//!         #[call_type(unchecked)]
//!         pub extern "jni" fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
//!             input1.push(input2);
//!             input1.iter().map(ToString::to_string).collect()
//!         }
//!
//!         #[call_type(safe(exception_class = "java.lang.IllegalArgumentException", message = "invalid value"))]
//!         pub extern "jni" fn bar(foo: i32) -> i32 { foo }
//!     }
//! }
//! ```
//!

use std::convert::TryFrom;
use std::str::FromStr;

use jni::errors::ErrorKind;
use jni::objects::{JObject, JString, JValue};
use jni::signature::JavaType;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use jni::JNIEnv;
use paste::paste;

pub use safe::*;
pub use unchecked::*;

pub mod safe;
pub mod unchecked;

/// A trait for types that are ffi-safe to use with JNI. It is implemented for primitives, [JOBject](jni::objects::JObject) and [jobject](jni::sys::jobject).
/// Users that want automatic conversion should instead implement [FromJavaValue], [IntoJavaValue] and/or [TryFromJavaValue], [TryIntoJavaValue]
pub trait JavaValue<'env> {
    const SIG_TYPE: &'static str = "Ljava/lang/Object;";

    fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env>;

    fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self;
}

/// A trait for types that can provide a reference to a [JNIEnv](jni::JNIEnv) object. Used when generating Java methods that take a "self" parameter.
pub trait JNIEnvLink<'env> {
    fn get_env(&self) -> &JNIEnv<'env>;
}

macro_rules! jvalue_types {
    ($type:ty: $boxed:ident ($sig:ident) [$unbox_method:ident]) => {
        impl<'env> JavaValue<'env> for $type {
            const SIG_TYPE: &'static str = stringify!($sig);

            fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env> {
                env.call_static_method_unchecked(concat!("java/lang/", stringify!($boxed)),
                    (concat!("java/lang/", stringify!($boxed)), "valueOf", concat!(stringify!(($sig)), "Ljava/lang/", stringify!($boxed), ";")),
                    JavaType::from_str(concat!("java/lang/", stringify!($boxed))).unwrap(),
                    &[Into::into(self)]).unwrap().l().unwrap()
            }

            fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self {
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

impl<'env> JavaValue<'env> for () {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        panic!("called `JavaValue::autobox` on unit value")
    }

    fn unbox(_s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {}
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
    const SIG_TYPE: &'static str = "Ljava/lang/String;";

    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        Into::into(self)
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        From::from(s)
    }
}

pub struct JValueWrapper<'a>(JValue<'a>);

impl<'a> From<JValue<'a>> for JValueWrapper<'a> {
    fn from(v: JValue<'a>) -> Self {
        JValueWrapper(v)
    }
}

impl<'a> From<JValueWrapper<'a>> for JValue<'a> {
    fn from(v: JValueWrapper<'a>) -> Self {
        v.0
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jboolean {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Bool(b) => Ok(b),
            _ => Err(ErrorKind::WrongJValueType("bool", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jbyte {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Byte(b) => Ok(b),
            _ => Err(ErrorKind::WrongJValueType("byte", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jchar {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Char(c) => Ok(c),
            _ => Err(ErrorKind::WrongJValueType("char", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jdouble {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Double(d) => Ok(d),
            _ => Err(ErrorKind::WrongJValueType("double", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jfloat {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Float(f) => Ok(f),
            _ => Err(ErrorKind::WrongJValueType("float", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jint {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Int(i) => Ok(i),
            _ => Err(ErrorKind::WrongJValueType("int", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jshort {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Short(s) => Ok(s),
            _ => Err(ErrorKind::WrongJValueType("short", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for jlong {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Long(l) => Ok(l),
            _ => Err(ErrorKind::WrongJValueType("long", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for () {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Void => Ok(()),
            _ => Err(ErrorKind::WrongJValueType("void", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for JObject<'a> {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Object(o) => Ok(o),
            _ => Err(ErrorKind::WrongJValueType("object", value.0.type_name()).into()),
        }
    }
}

impl<'a> TryFrom<JValueWrapper<'a>> for JString<'a> {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Object(o) => Ok(From::from(o)),
            _ => Err(ErrorKind::WrongJValueType("string", value.0.type_name()).into()),
        }
    }
}

pub trait Signature {
    const SIG_TYPE: &'static str;
}

impl<'e, T> Signature for T
where
    T: JavaValue<'e>,
{
    const SIG_TYPE: &'static str = T::SIG_TYPE;
}
