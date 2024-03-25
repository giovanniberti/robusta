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
//!         pub extern "jni" fn bar(foo: i32) -> ::robusta_jni::jni::errors::Result<i32> { Ok(foo) }
//!     }
//! }
//! ```
//!
//! # Raising exceptions from native code
//! If you want to have the option of throwing a Java exception from native code (conversion errors aside), you can
//! annotate your function signature with a [`jni::errors::Result<T>`] return type.
//!
//! When used with `#[call_type(safe)]`, if an `Err` is returned a Java exception is thrown (the one specified in the `call_type` attribute,
//! or `java.lang.RuntimeException` if omitted).
//!

use std::convert::TryFrom;
use std::str::FromStr;

use jni::errors::Error;
use jni::objects::{JObject, JValue, JList, JMap};
use jni::sys::jobject;

// For duplicate_item blocks
use jni::objects::{JString, JClass, JByteBuffer, JThrowable};
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};

use jni::JNIEnv;
use duplicate::duplicate_item;

pub use field::*;
pub use robusta_codegen::Signature;
pub use robusta_codegen::ArrSignature;
pub use safe::*;
pub use unchecked::*;
pub use local::*;

pub mod field;
pub mod safe;
pub mod unchecked;
pub mod local;

/// A trait for types that are ffi-safe to use with JNI. It is implemented for primitives, [JObject](jni::objects::JObject) and [jobject](jni::sys::jobject).
/// Users that want automatic conversion should instead implement [FromJavaValue], [IntoJavaValue] and/or [TryFromJavaValue], [TryIntoJavaValue]
pub trait JavaValue<'env> {
    /// Convert instance to a [`JObject`].
    fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env>;

    /// Convert [`JObject`] to the implementing type.
    fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self;
}

/// This trait provides [type signatures](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) for types.
/// It is necessary to support conversions to/from Java types.
///
/// While you can implement this trait manually, you should probably use the derive macro.
///
/// The derive macro requires a `#[package()]` attribute on implementing structs (most likely you already have that).
///
pub trait Signature {
    /// [Java type signature](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) for the implementing type.
    const SIG_TYPE: &'static str;
}

// TODO: This still is not gonna work well for multi-dimensional arrays, like Box<[Box<[T]>]>
pub trait ArrSignature {
    const ARR_SIG_TYPE: &'static str;
}

pub struct JValueWrapper<'a>(pub JValue<'a>);

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

impl<'env> Signature for JList<'env, 'env> {
    const SIG_TYPE: &'static str = "Ljava/util/List;";
}

impl<'env> Signature for JMap<'env, 'env> {
    const SIG_TYPE: &'static str = "Ljava/util/Map;";
}

impl<T: Signature> Signature for jni::errors::Result<T> {
    const SIG_TYPE: &'static str = <T as Signature>::SIG_TYPE;
}

impl<T: Signature> Signature for Option<T> {
    const SIG_TYPE: &'static str = <T as Signature>::SIG_TYPE;
}

impl<T: ArrSignature> ArrSignature for Option<T> {
    const ARR_SIG_TYPE: &'static str = <T as ArrSignature>::ARR_SIG_TYPE;
}

impl<T: ArrSignature> ArrSignature for jni::errors::Result<T> {
    const ARR_SIG_TYPE: &'static str = <T as ArrSignature>::ARR_SIG_TYPE;
}

impl<T: ArrSignature> Signature for Box<[T]> {
    const SIG_TYPE: &'static str = <T as ArrSignature>::ARR_SIG_TYPE;
}

// Similar to jvalue_types, but still different
impl<'env> JavaValue<'env> for jobject {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        unsafe { JObject::from_raw(self) }
    }

    fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
        s.into_raw()
    }
}

// Impls for (), similar to jvalue_types, but still different
impl Signature for () {
    const SIG_TYPE: &'static str = "V";
}

impl<'env> JavaValue<'env> for () {
    fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
        panic!("called `JavaValue::autobox` on unit value")
    }

    fn unbox(_s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {}
}

impl<'a> TryFrom<JValueWrapper<'a>> for () {
    type Error = jni::errors::Error;

    fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            JValue::Void => Ok(()),
            _ => Err(Error::WrongJValueType("void", value.0.type_name()).into()),
        }
    }
}

#[duplicate_item(
j_type boxed sig unbox_method j_val_type j_val_type_name;
[jboolean] [Boolean]    [Z]   [booleanValue]    [Bool]      ["bool"];
[jbyte]    [Byte]       [B]   [byteValue]       [Byte]      ["byte"];
[jchar]    [Character]  [C]   [charValue]       [Char]      ["char"];
[jdouble]  [Double]     [D]   [doubleValue]     [Double]    ["double"];
[jfloat]   [Float]      [F]   [floatValue]      [Float]     ["float"];
[jint]     [Integer]    [I]   [intValue]        [Int]       ["int"];
[jlong]    [Long]       [J]   [longValue]       [Long]      ["long"];
[jshort]   [Short]      [S]   [shortValue]      [Short]     ["short"];
)]
mod jvalue_types {
    use crate::convert::*;
    use paste::paste;
    use jni::signature::ReturnType;

    impl Signature for j_type {
        const SIG_TYPE: &'static str = stringify!(sig);
    }

    impl<'env> JavaValue<'env> for j_type {
        fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env> {
            env.call_static_method_unchecked(concat!("java/lang/", stringify!(boxed)),
                                             (concat!("java/lang/", stringify!(boxed)), "valueOf", concat!(stringify!((sig)), "Ljava/lang/", stringify!(boxed), ";")),
                                             ReturnType::from_str(concat!("Ljava/lang/", stringify!(boxed), ";")).unwrap(),
                                             &[JValue::from(self).to_jni()]).unwrap().l().unwrap()
        }

        fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self {
            paste!(Into::into(env.call_method_unchecked(s, (concat!("java/lang/", stringify!(boxed)), stringify!(unbox_method), concat!("()", stringify!(sig))), ReturnType::from_str(stringify!(sig)).unwrap(), &[])
                    .unwrap().[<sig:lower>]()
                    .unwrap()))
        }
    }

    //// Introduced in new version of jni-rs, pls keep and uncomment after migration
    // impl<'env> Signature for paste!([<J boxed Array>])<'env> {
    //     const SIG_TYPE: &'static str = concat!("[", stringify!(sig));
    // }

    // impl<'env> JavaValue<'env> for paste!([<J boxed Array>])<'env> {
    //     fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
    //         Into::into(self)
    //     }

    //     fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
    //         From::from(s)
    //     }
    // }

    impl<'a> TryFrom<JValueWrapper<'a>> for j_type {
        type Error = jni::errors::Error;

        fn try_from(value: JValueWrapper<'a>) -> Result<Self, Self::Error> {
            match value.0 {
                JValue::j_val_type(b) => Ok(b),
                _ => Err(Error::WrongJValueType(j_val_type_name, value.0.type_name()).into()),
            }
        }
    }
}

#[duplicate_item(
module_disambiguation j_type sig name;
[a] [JString < 'env >]      ["Ljava/lang/String;"]     ["string"];
[b] [JClass < 'env >]       ["Ljava/lang/Class;"]      ["class"];
[c] [JByteBuffer < 'env >]  ["Ljava/nio/ByteBuffer;"]  ["bytebuffer"];
//// Introduced in new version of jni-rs, pls keep and uncomment after migration
// [d] [JObjectArray < 'env >] ["[Ljava/lang/Object;"]    ["object_array"];
[e] [JThrowable < 'env >]   ["Ljava/lang/Throwable;"]  ["throwable"];
[f] [JObject < 'env >]      ["Ljava/lang/Object;"]     ["object"];
)]
mod jobject_types {
    use crate::convert::*;

    impl<'env> Signature for j_type {
        const SIG_TYPE: &'static str = sig;
    }

    // I believe it will be optimized away for JObject
    impl<'env> JavaValue<'env> for j_type {
        fn autobox(self, _env: &JNIEnv<'env>) -> JObject<'env> {
            Into::into(self)
        }

        fn unbox(s: JObject<'env>, _env: &JNIEnv<'env>) -> Self {
            From::from(s)
        }
    }

    impl<'env> TryFrom<JValueWrapper<'env>> for j_type {
        type Error = jni::errors::Error;

        fn try_from(value: JValueWrapper<'env>) -> Result<Self, Self::Error> {
            match value.0 {
                JValue::Object(o) => Ok(From::from(o)),
                _ => Err(Error::WrongJValueType(name, value.0.type_name()).into()),
            }
        }
    }
}

#[duplicate_item(
module_disambiguation j_type l_type;
[a] [JString] [JString < 'env >];
[b] [String] [String];
[c] [JClass] [JClass < 'env >];
[d] [JByteBuffer] [JByteBuffer < 'env >];
// TODO: Enable after migration
// [e] [JObjectArray] [JObjectArray<'env>];
// [f] [JBooleanArray] [JBooleanArray<'env>];
// [g] [JByteArray] [JByteArray<'env>];
// [h] [JCharacterArray] [JCharacterArray<'env>];
// [i] [JDoubleArray] [JDoubleArray<'env>];
// [j] [JFloatArray] [JFloatArray<'env>];
// [k] [JIntegerArray] [JIntegerArray<'env>];
// [l] [JLongArray] [JLongArray<'env>];
// [m] [JShortArray] [JShortArray<'env>];
[n] [JThrowable] [JThrowable < 'env >];
)]
mod arr_sign_impl {
    use crate::convert::*;

    impl<'env> ArrSignature for l_type {
        const ARR_SIG_TYPE: &'static str = constcat::concat!("[", <j_type as Signature>::SIG_TYPE);
    }
}
