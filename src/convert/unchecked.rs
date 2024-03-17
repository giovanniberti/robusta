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

use std::ptr::slice_from_raw_parts;
use jni::objects::{JList, JObject, JString, JValue, JClass, JByteBuffer, JThrowable};
use jni::sys::{jboolean, jbyte, jchar, jsize};
use jni::sys::{JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

use crate::convert::{ArrSignature, JavaValue, Signature};

pub use robusta_codegen::{FromJavaValue, IntoJavaValue};

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
    type Target = JString<'env>;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        env.new_string(self).unwrap()
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
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    }
}

impl Signature for bool {
    const SIG_TYPE: &'static str = <jboolean as Signature>::SIG_TYPE;
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for bool {
    type Source = jboolean;

    fn from(s: Self::Source, _env: &JNIEnv<'env>) -> Self {
        s == JNI_TRUE
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
        std::char::decode_utf16(std::iter::once(s))
            .next()
            .unwrap()
            .unwrap()
    }
}

impl ArrSignature for bool {
    const ARR_SIG_TYPE: &'static str = constcat::concat!("[", <bool as Signature>::SIG_TYPE);
}

impl<'env> IntoJavaValue<'env> for Box<[bool]> {
    type Target = JObject<'env>;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        let len = self.len();
        let buf: Vec<_> = self.iter().map(|&b| Into::into(b)).collect();
        let raw = env.new_boolean_array(len as i32).unwrap();
        env.set_boolean_array_region(raw, 0, &buf).unwrap();
        unsafe { Self::Target::from_raw(raw) }
    }
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for Box<[bool]> {
    type Source = JObject<'env>;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        let len = env.get_array_length(s.into_raw()).unwrap();
        let mut buf = vec![JNI_FALSE; len as usize].into_boxed_slice();
        env.get_boolean_array_region(s.into_raw(), 0, &mut *buf).unwrap();

        buf.iter().map(|&b| FromJavaValue::from(b, &env)).collect()
    }
}

impl<T> Signature for Vec<T> {
    const SIG_TYPE: &'static str = "Ljava/util/List;";
}

impl<'env, T> IntoJavaValue<'env> for Vec<T>
where
    T: IntoJavaValue<'env>,
{
    type Target = JObject<'env>;

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

        list.into()
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

impl ArrSignature for i8 {
    const ARR_SIG_TYPE: &'static str = constcat::concat!("[", <jbyte as Signature>::SIG_TYPE);
}

impl<'env> IntoJavaValue<'env> for Box<[i8]> {
    type Target = JObject<'env>;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        let conv = unsafe { &*slice_from_raw_parts(self.as_ref().as_ptr() as *const u8, self.as_ref().len()) };
        unsafe { Self::Target::from_raw(env.byte_array_from_slice(conv).unwrap()) }
    }
}

impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for Box<[i8]> {
    type Source = JObject<'env>;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        let buf = env.convert_byte_array(s.into_raw()).unwrap();
        let boxed_slice = buf.into_boxed_slice();
        let conv = unsafe { &*slice_from_raw_parts(boxed_slice.as_ref().as_ptr() as *const i8, boxed_slice.as_ref().len()) };
        conv.into()
    }
}

impl<'env, T> IntoJavaValue<'env> for jni::errors::Result<T>
where
    T: IntoJavaValue<'env>,
{
    type Target = <T as IntoJavaValue<'env>>::Target;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        self.map(|s| IntoJavaValue::into(s, env)).unwrap()
    }
}

impl<'env: 'borrow, 'borrow, T> FromJavaValue<'env, 'borrow> for Option<T>
where
    T: FromJavaValue<'env, 'borrow>,
    <T as FromJavaValue<'env, 'borrow>>::Source: Into<JObject<'env>> + Clone,
{
    type Source = <T as FromJavaValue<'env, 'borrow>>::Source;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        if env.is_same_object(s.clone().into(), JObject::null()).unwrap() {
            None
        } else { Some(T::from(s, env)) }
    }
}

impl<'env, T> IntoJavaValue<'env> for Option<T>
where
    T: IntoJavaValue<'env>,
    // It's possible to replace this with
    // <T as IntoJavaValue<'env>>::Target: Default,
    // after migration, so it'll work with primitive types too
    // (not sure if it break things for types with Target != JObject
    <T as IntoJavaValue<'env>>::Target: From<JObject<'env>>,
{
    type Target = <T as IntoJavaValue<'env>>::Target;
    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        match self {
            None => { From::from(JObject::null()) }
            Some(value) => { T::into(value, env) }
        }
    }
}

// TODO: Is there any way to impl it for Box<[T]> where T: Signature?
impl<'env> ArrSignature for JObject<'env> {
    const ARR_SIG_TYPE: &'static str = constcat::concat!("[", <JObject as Signature>::SIG_TYPE);
}

impl<'env: 'borrow, 'borrow, T> FromJavaValue<'env, 'borrow> for Box<[T]>
    where
        Box<[T]>: Signature,
        T: FromJavaValue<'env, 'borrow, Source = JObject<'env>>,
{
    // TODO: Replace with JObjectArray after migration to 0.21
    type Source = JObject<'env>;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        let len = env.get_array_length(s.into_raw()).unwrap();
        let mut buf = Vec::with_capacity(len as usize);
        for idx in 0..len {
            buf.push(env.get_object_array_element(s.into_raw(), idx).unwrap());
        }

        buf.into_boxed_slice().iter()
            .map(|&b| T::from(Into::into(b), &env))
            .collect()
    }
}

impl<'env, T> IntoJavaValue<'env> for Box<[T]>
    where
        Box<[T]>: Signature,
        T: IntoJavaValue<'env, Target = JObject<'env>>,
{
    // TODO: Replace with JObjectArray after migration to 0.21
    type Target = JObject<'env>;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        let vec = self.into_vec();
        let raw = env.new_object_array(
            vec.len() as jsize, <T as Signature>::SIG_TYPE, JObject::null()
        ).unwrap();
        for (idx, elem) in vec.into_iter().enumerate() {
            env.set_object_array_element(raw, idx as jsize, T::into(elem, env)).unwrap();
        }
        unsafe { Self::Target::from_raw(raw) }
    }
}

macro_rules! box_impl_unchecked {
    ($type:ty, $l_type:ty) => {
        impl<'env> ArrSignature for $l_type {
            const ARR_SIG_TYPE: &'static str = constcat::concat!("[", <$type as Signature>::SIG_TYPE);
        }

        impl<'env: 'borrow, 'borrow> FromJavaValue<'env, 'borrow> for Box<[$l_type]>
        {
            // TODO: Replace with JObjectArray after migration to 0.21
            type Source = JObject<'env>;

            fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
                let len = env.get_array_length(s.into_raw()).unwrap();
                let mut buf = Vec::with_capacity(len as usize);
                for idx in 0..len {
                    buf.push(env.get_object_array_element(s.into_raw(), idx).unwrap());
                }

                buf.into_boxed_slice().iter()
                .map(|&b| <$type as FromJavaValue>::from(Into::into(b), &env))
                .collect()
            }
        }

        impl<'env> IntoJavaValue<'env> for Box<[$l_type]>
        {
            // TODO: Replace with JObjectArray after migration to 0.21
            type Target = JObject<'env>;

            fn into(self, env: &JNIEnv<'env>) -> Self::Target {
                let vec = self.into_vec();
                let raw = env.new_object_array(
                    vec.len() as jsize, <$type as Signature>::SIG_TYPE, JObject::null()
                ).unwrap();
                for (idx, elem) in vec.into_iter().enumerate() {
                    env.set_object_array_element(raw, idx as jsize, <$type as IntoJavaValue>::into(elem, env)).unwrap();
                }
                unsafe { Self::Target::from_raw(raw) }
            }
        }
    };

    ($type:ty, $l_type:ty, $($rest:ty, $l_rest_boxed:ty),+) => {
        box_impl_unchecked!($type, $l_type);

        box_impl_unchecked!($($rest, $l_rest_boxed),+);
    }
}

box_impl_unchecked! {
    JString, JString<'env>,
    String, String,
    JClass, JClass<'env>,
    JByteBuffer, JByteBuffer<'env>,
    // TODO: Enable after migration
    // JObjectArray, JObjectArray<'env>,
    // JBooleanArray, JBooleanArray<'env>
    // JByteArray, JByteArray<'env>
    // JCharacterArray, JCharacterArray<'env>
    // JDoubleArray, JDoubleArray<'env>
    // JFloatArray, JFloatArray<'env>
    // JIntegerArray, JIntegerArray<'env>
    // JLongArray, JLongArray<'env>
    // JShortArray, JShortArray<'env>
    JThrowable, JThrowable<'env>
}
