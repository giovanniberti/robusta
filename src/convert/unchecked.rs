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
use jni::objects::{JList, JObject, JString, JValue};
use jni::objects::{JClass, JByteBuffer, JThrowable};
use jni::sys::{jboolean, jbyte, jchar, jsize};
use jni::sys::{JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

use crate::convert::{ArrSignature, JavaValue, Signature};

pub use robusta_codegen::{FromJavaValue, IntoJavaValue};

use duplicate::duplicate_item;

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

pub trait BoxedFromJavaValue<'env: 'borrow, 'borrow, T, U>
    where
        T: FromJavaValue<'env, 'borrow, Source=U>,
        Box<[T]>: Signature,
{
    type Source: JavaValue<'env>;
    const SIG_TYPE: &'static str = <Box<[T]> as Signature>::SIG_TYPE;

    fn boxed_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Box<[T]>;
}

pub trait BoxedIntoJavaValue<'env, T, U>
    where
        T: IntoJavaValue<'env, Target=U>,
        Box<[T]>: Signature,
{
    type Target: JavaValue<'env>;
    const SIG_TYPE: &'static str = <Box<[T]> as Signature>::SIG_TYPE;

    fn boxed_into(t: Box<[T]>, env: &JNIEnv<'env>) -> Self::Target;
}

impl<'env: 'borrow, 'borrow> BoxedFromJavaValue<'env, 'borrow, bool, <bool as FromJavaValue<'env, 'borrow>>::Source> for (bool, <bool as FromJavaValue<'env, 'borrow>>::Source) {
    type Source = JObject<'env>;

    fn boxed_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Box<[bool]> {
        let len = env.get_array_length(s.into_raw()).unwrap();
        let mut buf = vec![JNI_FALSE; len as usize].into_boxed_slice();
        env.get_boolean_array_region(s.into_raw(), 0, &mut *buf).unwrap();

        buf.iter().map(|&b| FromJavaValue::from(b, &env)).collect()
    }
}

impl<'env> BoxedIntoJavaValue<'env, bool, <bool as IntoJavaValue<'env>>::Target> for (bool, <bool as IntoJavaValue<'env>>::Target) {
    type Target = JObject<'env>;

    fn boxed_into(t: Box<[bool]>, env: &JNIEnv<'env>) -> Self::Target {
        let len = t.len();
        let buf: Vec<_> = t.iter().map(|&b| Into::into(b)).collect();
        let raw = env.new_boolean_array(len as i32).unwrap();
        env.set_boolean_array_region(raw, 0, &buf).unwrap();
        unsafe { Self::Target::from_raw(raw) }
    }
}

impl<'env: 'borrow, 'borrow> BoxedFromJavaValue<'env, 'borrow, i8, <i8 as FromJavaValue<'env, 'borrow>>::Source> for (i8, <i8 as FromJavaValue<'env, 'borrow>>::Source) {
    type Source = JObject<'env>;

    fn boxed_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Box<[i8]> {
        let buf = env.convert_byte_array(s.into_raw()).unwrap();
        let boxed_slice = buf.into_boxed_slice();
        let conv = unsafe { &*slice_from_raw_parts(boxed_slice.as_ref().as_ptr() as *const i8, boxed_slice.as_ref().len()) };
        conv.into()
    }
}

impl<'env> BoxedIntoJavaValue<'env, i8, <i8 as IntoJavaValue<'env>>::Target> for (i8, <i8 as IntoJavaValue<'env>>::Target) {
    type Target = JObject<'env>;

    fn boxed_into(t: Box<[i8]>, env: &JNIEnv<'env>) -> Self::Target {
        let conv = unsafe { &*slice_from_raw_parts(t.as_ref().as_ptr() as *const u8, t.as_ref().len()) };
        unsafe { Self::Target::from_raw(env.byte_array_from_slice(conv).unwrap()) }
    }
}

#[duplicate_item(
module_disambiguation j_type;
[a] [JObject];
[b] [JString];
[c] [JClass];
[d] [JByteBuffer];
// TODO: Enable after migration
// [e] [JObjectArray];
// [f] [JBooleanArray];
// [g] [JByteArray];
// [h] [JCharacterArray];
// [i] [JDoubleArray];
// [j] [JFloatArray];
// [k] [JIntegerArray];
// [l] [JLongArray];
// [m] [JShortArray];
[n] [JThrowable];
)]
mod box_impl {
    use crate::convert::unchecked::*;
    impl<'env: 'borrow, 'borrow, T> BoxedFromJavaValue<'env, 'borrow, T, j_type<'env>> for (T, j_type<'env>)
        where
            Box<[T]>: Signature,
            T: FromJavaValue<'env, 'borrow, Source = j_type<'env>>,
    {
        // TODO: Replace with JObjectArray after migration to 0.21
        type Source = JObject<'env>;

        fn boxed_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Box<[T]> {
            let len = env.get_array_length(s.into_raw()).unwrap();
            let mut buf = Vec::with_capacity(len as usize);
            for idx in 0..len {
                // TODO: use AutoLocal - and convert immediately there - for types that
                // don't hold local ref, so env.delete_local_ref is safe
                buf.push(env.get_object_array_element(s.into_raw(), idx).unwrap());
            }

            buf.into_boxed_slice().iter()
                .map(|&b| T::from(Into::into(b), &env))
                .collect()
        }
    }

    impl<'env, T> BoxedIntoJavaValue<'env, T, j_type<'env>> for (T, j_type<'env>)
        where
            Box<[T]>: Signature,
            T: IntoJavaValue<'env, Target = j_type<'env>>,
    {
        // TODO: Replace with JObjectArray after migration to 0.21
        type Target = JObject<'env>;

        fn boxed_into(t: Box<[T]>, env: &JNIEnv<'env>) -> Self::Target {
            let vec = t.into_vec();
            let raw = env.new_object_array(
                vec.len() as jsize, <T as Signature>::SIG_TYPE, JObject::null()
            ).unwrap();
            for (idx, elem) in vec.into_iter().enumerate() {
                // TODO: use AutoLocal - and convert immediately there - for types that
                // don't hold local ref, so env.delete_local_ref is safe
                env.set_object_array_element(raw, idx as jsize, T::into(elem, env)).unwrap();
            }
            unsafe { Self::Target::from_raw(raw) }
        }
    }
}

impl<'env: 'borrow, 'borrow, T> FromJavaValue<'env, 'borrow> for Box<[T]>
    where
        T: FromJavaValue<'env, 'borrow>,
        Box<[T]>: Signature,
        (T, <T as FromJavaValue<'env, 'borrow>>::Source): BoxedFromJavaValue<'env, 'borrow, T, <T as FromJavaValue<'env, 'borrow>>::Source>,
{
    type Source = <(T, <T as FromJavaValue<'env, 'borrow>>::Source) as BoxedFromJavaValue<'env, 'borrow, T, <T as FromJavaValue<'env, 'borrow>>::Source>>::Source;

    fn from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Self {
        <(T, <T as FromJavaValue<'env, 'borrow>>::Source) as BoxedFromJavaValue<T, <T as FromJavaValue<'env, 'borrow>>::Source>>::boxed_from(s, env)
    }
}

impl<'env, T> IntoJavaValue<'env> for Box<[T]>
    where
        T: IntoJavaValue<'env>,
        Box<[T]>: Signature,
        (T, <T as IntoJavaValue<'env>>::Target): BoxedIntoJavaValue<'env, T, <T as IntoJavaValue<'env>>::Target>,
{
    type Target = <(T, <T as IntoJavaValue<'env>>::Target) as BoxedIntoJavaValue<'env, T, <T as IntoJavaValue<'env>>::Target>>::Target;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        <(T, <T as IntoJavaValue<'env>>::Target) as BoxedIntoJavaValue<T, <T as IntoJavaValue<'env>>::Target>>::boxed_into(self, env)
    }
}

impl ArrSignature for bool {
    const ARR_SIG_TYPE: &'static str = constcat::concat!("[", <bool as Signature>::SIG_TYPE);
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
    <T as IntoJavaValue<'env>>::Target: Default,
{
    type Target = <T as IntoJavaValue<'env>>::Target;
    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        match self {
            None => { Self::Target::default() }
            Some(value) => { T::into(value, env) }
        }
    }
}

// TODO: Is there any way to impl it for Box<[T]> where T: Signature?
impl<'env> ArrSignature for JObject<'env> {
    const ARR_SIG_TYPE: &'static str = constcat::concat!("[", <JObject as Signature>::SIG_TYPE);
}
