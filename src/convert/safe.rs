//! Fallible conversions traits.
//!
//! These are the traits selected if `call_type` is omitted or if specified with a `safe` parameter.
//!
//! ```ignore
//! #[call_type(safe)]
//! ```
//!
//! If any conversion fails, e.g. while converting input parameters or return arguments a Java exception is thrown.
//! Exception class and exception message can be customized with the `exception_class` and `message` parameters of the `safe` option, as such:
//!
//! ```ignore
//! #[call_type(safe(exception_class = "java.io.IOException", message = "Error while calling JNI function!"))]
//! ```
//!
//! Both of these parameters are optional. By default, the exception class is `java.lang.RuntimeException`.
//!

use std::ptr::slice_from_raw_parts;
use jni::errors::{Error, Result};
use jni::objects::{JList, JObject, JString, JValue};
use jni::objects::{JClass, JByteBuffer, JThrowable};
use jni::sys::{jboolean, jchar, jsize};
use jni::sys::JNI_FALSE;
use jni::JNIEnv;

use crate::convert::unchecked::{FromJavaValue, IntoJavaValue};
use crate::convert::{JavaValue, Signature};

pub use robusta_codegen::{TryFromJavaValue, TryIntoJavaValue};

use duplicate::duplicate_item;

/// Conversion trait from Rust values to Java values, analogous to [TryInto](std::convert::TryInto). Used when converting types returned from JNI-available functions.
///
/// This is the default trait used when converting values from Rust to Java.
///
/// # Notes on derive macro
/// The same notes on [`TryFromJavaValue`] apply.
///
/// Note that when autoderiving `TryIntoJavaValue` for `T`, an implementation for all of `T`, `&T` and `&mut T` is generated (for ergonomics).
///
pub trait TryIntoJavaValue<'env>: Signature {
    /// Conversion target type.
    type Target: JavaValue<'env>;

    /// [Signature](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) of the source type.
    /// By default, use the one defined on the [`Signature`] trait for the implementing type.
    const SIG_TYPE: &'static str = <Self as Signature>::SIG_TYPE;

    /// Perform the conversion.
    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target>;
}

/// Conversion trait from Java values to Rust values, analogous to [TryFrom](std::convert::TryInto). Used when converting types that are input to JNI-available functions.
///
/// This is the default trait used when converting values from Java to Rust.
///
/// # Notes on the derive macro
/// When using the derive macro, the deriving struct **must** have a [`Local`] field annotated with both `'env` and `'borrow` lifetimes and a `#[instance]` attribute.
/// This fields keeps a [local reference](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/design.html#global-and-local-references) to the underlying Java object.
/// All other fields are automatically initialized from fields on the Java instance with the same name.
///
/// Example:
///
/// ```rust
/// # use robusta_jni::bridge;
/// use robusta_jni::convert::{Signature, TryFromJavaValue, Local};
/// #
/// # #[bridge]
/// # mod jni {
///     # use robusta_jni::convert::{Signature, TryFromJavaValue, Local};
///     # use robusta_jni::jni::JNIEnv;
///     # use jni::objects::JObject;
///
/// #[derive(Signature, TryFromJavaValue)]
/// #[package()]
/// struct A<'env: 'borrow, 'borrow> {
///     #[instance]
///     raw: Local<'env, 'borrow>,
///     foo: i32
/// }
/// # }
/// ```
///
/// [`Local`]: convert::Local
///
pub trait TryFromJavaValue<'env: 'borrow, 'borrow>
where
    Self: Sized + Signature,
{
    /// Conversion source type.
    type Source: JavaValue<'env>;

    /// [Signature](https://docs.oracle.com/en/java/javase/15/docs/specs/jni/types.html#type-signatures) of the target type.
    /// By default, use the one defined on the [`Signature`] trait for the implementing type.
    const SIG_TYPE: &'static str = <Self as Signature>::SIG_TYPE;

    /// Perform the conversion.
    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self>;
}

impl<'env, T> TryIntoJavaValue<'env> for T
where
    T: JavaValue<'env> + Signature,
{
    type Target = T;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        Ok(IntoJavaValue::into(self, env))
    }
}

impl<'env: 'borrow, 'borrow, T> TryFromJavaValue<'env, 'borrow> for T
where
    T: JavaValue<'env> + Signature,
{
    type Source = T;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        Ok(FromJavaValue::from(s, env))
    }
}

impl<'env> TryIntoJavaValue<'env> for String {
    type Target = JString<'env>;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        env.new_string(self)
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for String {
    type Source = JString<'env>;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        env.get_string(s).map(Into::into)
    }
}

impl<'env> TryIntoJavaValue<'env> for bool {
    type Target = jboolean;

    fn try_into(self, _env: &JNIEnv<'env>) -> Result<Self::Target> {
        Ok(IntoJavaValue::into(self, _env))
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for bool {
    type Source = jboolean;

    fn try_from(s: Self::Source, _env: &JNIEnv<'env>) -> Result<Self> {
        Ok(FromJavaValue::from(s, _env))
    }
}

impl<'env> TryIntoJavaValue<'env> for char {
    type Target = jchar;

    fn try_into(self, _env: &JNIEnv<'env>) -> Result<Self::Target> {
        Ok(IntoJavaValue::into(self, _env))
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for char {
    type Source = jchar;

    fn try_from(s: Self::Source, _env: &JNIEnv<'env>) -> Result<Self> {
        let res = std::char::decode_utf16(std::iter::once(s)).next();

        match res {
            Some(Ok(c)) => Ok(c),
            Some(Err(_)) | None => Err(Error::WrongJValueType("char", "jchar")),
        }
    }
}

pub trait BoxedTryFromJavaValue<'env: 'borrow, 'borrow, T, U>
where
    T: TryFromJavaValue<'env, 'borrow, Source=U>,
    Box<[T]>: Signature,
{
    type Source: JavaValue<'env>;
    const SIG_TYPE: &'static str = <Box<[T]> as Signature>::SIG_TYPE;

    fn boxed_try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Box<[T]>>;
}

pub trait BoxedTryIntoJavaValue<'env, T, U>
    where
        T: TryIntoJavaValue<'env, Target=U>,
        Box<[T]>: Signature,
{
    type Target: JavaValue<'env>;
    const SIG_TYPE: &'static str = <Box<[T]> as Signature>::SIG_TYPE;

    fn boxed_try_into(t: Box<[T]>, env: &JNIEnv<'env>) -> Result<Self::Target>;
}

impl<'env: 'borrow, 'borrow> BoxedTryFromJavaValue<'env, 'borrow, bool, <bool as TryFromJavaValue<'env, 'borrow>>::Source> for (bool, <bool as TryFromJavaValue<'env, 'borrow>>::Source) {
    type Source = JObject<'env>;

    fn boxed_try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Box<[bool]>> {
        let len = env.get_array_length(s.into_raw())?;
        let mut buf = vec![JNI_FALSE; len as usize].into_boxed_slice();
        env.get_boolean_array_region(s.into_raw(), 0, &mut *buf)?;

        buf.iter()
            .map(|&b| TryFromJavaValue::try_from(b, &env))
            .collect()
    }
}

impl<'env> BoxedTryIntoJavaValue<'env, bool, <bool as TryIntoJavaValue<'env>>::Target> for (bool, <bool as TryIntoJavaValue<'env>>::Target) {
    type Target = JObject<'env>;

    fn boxed_try_into(t: Box<[bool]>, env: &JNIEnv<'env>) -> Result<Self::Target> {
        let len = t.len();
        let buf: Vec<_> = t.iter().map(|&b| Into::into(b)).collect();
        let raw = env.new_boolean_array(len as i32)?;
        env.set_boolean_array_region(raw, 0, &buf)?;
        Ok(unsafe { Self::Target::from_raw(raw) })
    }
}

impl<'env: 'borrow, 'borrow> BoxedTryFromJavaValue<'env, 'borrow, i8, <i8 as TryFromJavaValue<'env, 'borrow>>::Source> for (i8, <i8 as TryFromJavaValue<'env, 'borrow>>::Source) {
    type Source = JObject<'env>;

    fn boxed_try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Box<[i8]>> {
        let buf = env.convert_byte_array(s.into_raw())?;
        let boxed_slice = buf.into_boxed_slice();
        let conv = unsafe { &*slice_from_raw_parts(boxed_slice.as_ref().as_ptr() as *const i8, boxed_slice.as_ref().len()) };
        Ok(conv.into())
    }
}

impl<'env> BoxedTryIntoJavaValue<'env, i8, <i8 as TryIntoJavaValue<'env>>::Target> for (i8, <i8 as TryIntoJavaValue<'env>>::Target) {
    type Target = JObject<'env>;

    fn boxed_try_into(t: Box<[i8]>, env: &JNIEnv<'env>) -> Result<Self::Target> {
        let conv = unsafe { &*slice_from_raw_parts(t.as_ref().as_ptr() as *const u8, t.as_ref().len()) };
        env.byte_array_from_slice(conv).map(|val| unsafe { Self::Target::from_raw(val) } )
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
    use crate::convert::safe::*;
    impl<'env: 'borrow, 'borrow, T> BoxedTryFromJavaValue<'env, 'borrow, T, j_type<'env>> for (T, j_type<'env>)
        where
            Box<[T]>: Signature,
            T: TryFromJavaValue<'env, 'borrow, Source = j_type<'env>>,
    {
        // TODO: Replace with JObjectArray after migration to 0.21
        type Source = JObject<'env>;

        fn boxed_try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Box<[T]>> {
            let len = env.get_array_length(s.into_raw())?;
            let mut buf = Vec::with_capacity(len as usize);
            for idx in 0..len {
                // TODO: use AutoLocal - and convert immediately there - for types that
                // don't hold local ref, so env.delete_local_ref is safe
                buf.push(env.get_object_array_element(s.into_raw(), idx)?);
            }

            buf.into_boxed_slice().iter()
                .map(|&b| T::try_from(Into::into(b), &env))
                .collect()
        }
    }

    impl<'env, T> BoxedTryIntoJavaValue<'env, T, j_type<'env>> for (T, j_type<'env>)
        where
            Box<[T]>: Signature,
            T: TryIntoJavaValue<'env, Target = j_type<'env>>,
    {
        // TODO: Replace with JObjectArray after migration to 0.21
        type Target = JObject<'env>;

        fn boxed_try_into(t: Box<[T]>, env: &JNIEnv<'env>) -> Result<Self::Target> {
            let vec = t.into_vec();
            let raw = env.new_object_array(
                vec.len() as jsize, <T as Signature>::SIG_TYPE, JObject::null()
            )?;
            for (idx, elem) in vec.into_iter().enumerate() {
                // TODO: use AutoLocal - and convert immediately there - for types that
                // don't hold local ref, so env.delete_local_ref is safe
                env.set_object_array_element(raw, idx as jsize, T::try_into(elem, env)?)?;
            }
            Ok(unsafe { Self::Target::from_raw(raw) })
        }
    }
}

// https://stackoverflow.com/questions/40392524/conflicting-trait-implementations-even-though-associated-types-differ/40408431#40408431
impl<'env: 'borrow, 'borrow, T> TryFromJavaValue<'env, 'borrow> for Box<[T]>
where
    T: TryFromJavaValue<'env, 'borrow>,
    Box<[T]>: Signature,
    (T, <T as TryFromJavaValue<'env, 'borrow>>::Source): BoxedTryFromJavaValue<'env, 'borrow, T, <T as TryFromJavaValue<'env, 'borrow>>::Source>,
{
    type Source = <(T, <T as TryFromJavaValue<'env, 'borrow>>::Source) as BoxedTryFromJavaValue<'env, 'borrow, T, <T as TryFromJavaValue<'env, 'borrow>>::Source>>::Source;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        <(T, <T as TryFromJavaValue<'env, 'borrow>>::Source) as BoxedTryFromJavaValue<T, <T as TryFromJavaValue<'env, 'borrow>>::Source>>::boxed_try_from(s, env)
    }
}

impl<'env, T> TryIntoJavaValue<'env> for Box<[T]>
    where
        T: TryIntoJavaValue<'env>,
        Box<[T]>: Signature,
        (T, <T as TryIntoJavaValue<'env>>::Target): BoxedTryIntoJavaValue<'env, T, <T as TryIntoJavaValue<'env>>::Target>,
{
    type Target = <(T, <T as TryIntoJavaValue<'env>>::Target) as BoxedTryIntoJavaValue<'env, T, <T as TryIntoJavaValue<'env>>::Target>>::Target;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        <(T, <T as TryIntoJavaValue<'env>>::Target) as BoxedTryIntoJavaValue<T, <T as TryIntoJavaValue<'env>>::Target>>::boxed_try_into(self, env)
    }
}

impl<'env, T> TryIntoJavaValue<'env> for Vec<T>
where
    T: TryIntoJavaValue<'env>,
{
    type Target = JObject<'env>;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        let obj = env.new_object(
            "java/util/ArrayList",
            "(I)V",
            &[JValue::Int(self.len() as i32)],
        )?;
        let list = JList::from_env(&env, obj)?;

        let _: Result<Vec<_>> = self
            .into_iter()
            .map::<Result<_>, _>(|el| {
                Ok(JavaValue::autobox(
                    TryIntoJavaValue::try_into(el, &env)?,
                    &env,
                ))
            })
            .map(|el| Ok(list.add(el?)))
            .collect();

        Ok(list.into())
    }
}

impl<'env: 'borrow, 'borrow, T, U> TryFromJavaValue<'env, 'borrow> for Vec<T>
where
    T: TryFromJavaValue<'env, 'borrow, Source = U>,
    U: JavaValue<'env>,
{
    type Source = JObject<'env>;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        let list = JList::from_env(env, s)?;

        list.iter()?
            .map(|el| T::try_from(U::unbox(el, env), env))
            .collect()
    }
}

/// When returning a [`jni::errors::Result`], if the returned variant is `Ok(v)` then the value `v` is returned as usual.
///
/// If the returned value is `Err`, the Java exception specified in the `#[call_type(safe)]` attribute is thrown
/// (by default `java.lang.RuntimeException`)
impl<'env, T> TryIntoJavaValue<'env> for jni::errors::Result<T>
where
    T: TryIntoJavaValue<'env>,
{
    type Target = <T as TryIntoJavaValue<'env>>::Target;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        self.and_then(|s| TryIntoJavaValue::try_into(s, env))
    }
}

impl<'env: 'borrow, 'borrow, T> TryFromJavaValue<'env, 'borrow> for Option<T>
where
    T: TryFromJavaValue<'env, 'borrow>,
    // TODO: Remove Clone after migration
    <T as TryFromJavaValue<'env, 'borrow>>::Source: Into<JObject<'env>> + Clone,
{
    type Source = <T as TryFromJavaValue<'env, 'borrow>>::Source;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        if env.is_same_object(s.clone().into(), JObject::null())? {
            Ok(None)
        } else { Ok(Some(T::try_from(s, env)?)) }
    }
}

impl<'env, T> TryIntoJavaValue<'env> for Option<T>
where
    T: TryIntoJavaValue<'env>,
    <T as TryIntoJavaValue<'env>>::Target: Default,
{
    type Target = <T as TryIntoJavaValue<'env>>::Target;
    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        match self {
            None => { Ok(Self::Target::default()) }
            Some(value) => { T::try_into(value, env) }
        }
    }
}
