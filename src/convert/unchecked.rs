//! Infallible conversion traits.
//!
//! These traits allow for a leaner generated glue code, with possibly some performance benefits.
//!
//! These conversion traits can be enabled to be used during code generation with the `unchecked` option on the `call_type` attribute, as so:
//!
//! ```ignore
//! #[call_type(unchecked)]
//! ```
//!
//! **These functions *will* panic should any conversion fail.**
//!

use jni::objects::{JList, JObject, JString, JValue};
use jni::sys::{jboolean, jbooleanArray, jchar, jobject, jstring};
use jni::JNIEnv;

use crate::convert::{JavaValue, Signature};

pub use robusta_codegen::{IntoJavaValue, FromJavaValue};

/// Conversion trait from Rust values to Java values, analogous to [Into]. Used when converting types returned from JNI-available functions.
///
/// The usage of this trait in the generated code can be enabled with the `#[call_type(unchecked)]` attribute on a per-method basis.
///
/// When using this trait the conversion is assumed to be infallible.
/// Should a conversion fail, a panic will be raised.
///
/// # Notes on the derive macro
///
/// The same notes on [`TryIntoJavaValue`] apply.
///
/// [`TryIntoJavaValue`]: crate::convert::TryIntoJavaValue
///
pub trait IntoJavaValue<'env>: Signature {
    /// Conversion target type.
    type Target: JavaValue<'env>;

    /// [Signature](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) of the source type.
    /// By default, use the one defined on the [`Signature`] trait for the implementing type.
    const SIG_TYPE: &'static str = <Self as Signature>::SIG_TYPE;

    /// Perform the conversion.
    fn into(self, env: &JNIEnv<'env>) -> Self::Target;
}

/// Conversion trait from Java values to Rust values, analogous to [From]. Used when converting types that are input to JNI-available functions.
///
/// # Notes on derive macro
///
/// The same notes on [`TryFromJavaValue`] apply.
///
/// [`TryFromJavaValue`]: crate::convert::TryFromJavaValue
///
pub trait FromJavaValue<'env: 'borrow, 'borrow>: Signature {
    /// Conversion source type.
    type Source: JavaValue<'env>;

    /// [Signature](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) of the target type.
    /// By default, use the one defined on the [`Signature`] trait for the implementing type.
    const SIG_TYPE: &'static str = <Self as Signature>::SIG_TYPE;

    /// Perform the conversion.
    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self;
}

impl<'env, T> IntoJavaValue<'env> for T
where
    T: JavaValue<'env> + Signature,
{
    type Target = T;

    fn into(self, _: &JNIEnv<'env>) -> Self::Target {
        self
    }
}

impl<'env: 'borrow, 'borrow, T> FromJavaValue<'env, 'borrow> for T
where
    T: JavaValue<'env> + Signature,
{
    type Source = T;

    fn from(t: Self::Source, _: &'borrow JNIEnv<'env>) -> Self {
        t
    }
}

impl Signature for String {
    const SIG_TYPE: &'static str = "Ljava/lang/String;";
}

impl<'env> IntoJavaValue<'env> for String {
    type Target = jstring;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        env.new_string(self).unwrap().into_inner()
    }
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for String {
    type Source = JString<'env>;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        env.get_string(s).unwrap().into()
    }
}

impl<'env> IntoJavaValue<'env> for bool {
    type Target = jboolean;

    fn into(self, _env: &JNIEnv<'env>) -> Self::Target {
        if self {
            1
        } else {
            0
        }
    }
}

impl Signature for bool {
    const SIG_TYPE: &'static str = <jboolean as Signature>::SIG_TYPE;
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for bool {
    type Source = jboolean;

    fn from(s: Self::Source, _env: &JNIEnv<'env>) -> Self {
        s == 1
    }
}

impl Signature for char {
    const SIG_TYPE: &'static str = <jchar as Signature>::SIG_TYPE;
}

impl<'env> IntoJavaValue<'env> for char {
    type Target = jchar;

    fn into(self, _env: &JNIEnv<'env>) -> Self::Target {
        self as jchar
    }
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for char {
    type Source = jchar;

    fn from(s: Self::Source, _env: &JNIEnv<'env>) -> Self {
        std::char::decode_utf16(std::iter::once(s)).next().unwrap().unwrap()
    }
}

impl<'env> IntoJavaValue<'env> for Box<[bool]> {
    type Target = jbooleanArray;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        let len = self.len();
        let buf: Vec<_> = self.iter().map(|&b| Into::into(b)).collect();
        let raw = env.new_boolean_array(len as i32).unwrap();
        env.set_boolean_array_region(raw, 0, &buf).unwrap();
        raw
    }
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for Box<[bool]> {
    type Source = jbooleanArray;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        let len = env.get_array_length(s).unwrap();
        let mut buf = Vec::with_capacity(len as usize).into_boxed_slice();
        env.get_boolean_array_region(s, 0, &mut *buf).unwrap();

        buf.iter().map(|&b| FromJavaValue::from(b, &env)).collect()
    }
}

impl<T> Signature for Vec<T> {
    const SIG_TYPE: &'static str = "Ljava/util/ArrayList;";
}

impl<'env, T> IntoJavaValue<'env> for Vec<T>
where
    T: IntoJavaValue<'env>,
{
    type Target = jobject;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        let obj = env
            .new_object(
                "java/util/ArrayList",
                "(I)V",
                &[JValue::Int(self.len() as i32)],
            )
            .unwrap();
        let list = JList::from_env(&env, obj).unwrap();

        self.into_iter()
            .map(|el| JavaValue::autobox(IntoJavaValue::into(el, &env), &env))
            .for_each(|el| {
                list.add(el).unwrap();
            });

        list.into_inner()
    }
}

impl<'env: 'borrow, 'borrow, T, U> FromJavaValue<'env, 'borrow> for Vec<T>
where
    T: FromJavaValue<'env, 'borrow, Source = U>,
    U: JavaValue<'env>,
{
    type Source = JObject<'env>;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        let list = JList::from_env(env, s).unwrap();

        list.iter()
            .unwrap()
            .map(|el| T::from(U::unbox(el, env), env))
            .collect()
    }
}

impl<'env, T> IntoJavaValue<'env> for jni::errors::Result<T>
where
    T: IntoJavaValue<'env>
{
    type Target = <T as IntoJavaValue<'env>>::Target;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        self.map(|s| IntoJavaValue::into(s, env)).unwrap()
    }
}
