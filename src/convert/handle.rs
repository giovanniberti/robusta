// TODO: Add `Signature` as `JavaValue` supertrait,
//  Replace all `SIG_TYPE` with `<T as Signature>::SIG_TYPE`
//  Use HandleDispatcher in codegen

use jni::objects::JObject;
use jni::JNIEnv;
use std::ops::{Deref, DerefMut};
use crate::convert::{TryFromJavaValue, FromJavaValue, JNIEnvLink, IntoJavaValue, TryIntoJavaValue, JavaValue};

pub trait Signature {
    const SIG_TYPE: &'static str;
}

pub struct Handle<'e, T> {
    raw: JObject<'e>,
    env: JNIEnv<'e>,
    inner: T,
}

impl<'e, T> Deref for Handle<'e, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'e, T> DerefMut for Handle<'e, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'e, T> Signature for Handle<'e, T>
where
    T: Signature,
{
    const SIG_TYPE: &'static str = T::SIG_TYPE;
}

impl<'env, T> TryFromJavaValue<'env> for Handle<'env, T>
where
    T: TryFromJavaValue<'env, Source = JObject<'env>>,
{
    type Source = JObject<'env>;

    fn try_from(s: Self::Source, env: &JNIEnv<'env>) -> jni::errors::Result<Self> {
        T::try_from(s, env).map(|inner| Handle {
            raw: s,
            env: env.clone(),
            inner,
        })
    }
}

impl<'env, T> FromJavaValue<'env> for Handle<'env, T>
where
    T: FromJavaValue<'env, Source = JObject<'env>>,
{
    type Source = JObject<'env>;

    fn from(s: Self::Source, env: &JNIEnv<'env>) -> Self {
        Handle {
            raw: s,
            env: env.clone(),
            inner: T::from(s, env),
        }
    }
}

impl<'e, T> JNIEnvLink<'e> for Handle<'e, T> {
    fn get_env(&self) -> &JNIEnv<'e> {
        &self.env
    }
}

impl<'e, T> IntoJavaValue<'e> for Handle<'e, T>
where
    T: Signature,
{
    type Target = JObject<'e>;
    const SIG_TYPE: &'static str = T::SIG_TYPE;

    fn into(self, _env: &JNIEnv<'e>) -> Self::Target {
        self.raw
    }
}

impl<'env, T> TryIntoJavaValue<'env> for Handle<'env, T>
where
    T: Signature,
{
    type Target = JObject<'env>;
    const SIG_TYPE: &'static str = T::SIG_TYPE;

    fn try_into(self, _env: &JNIEnv<'env>) -> jni::errors::Result<Self::Target> {
        Ok(self.raw)
    }
}

pub trait HandleDispatcher<'e> {
    /// This would be a `Signature + (FromJavaValue || TryFromJavaValue)` bound, but it is currently not expressible in Rust,
    /// so we opt for a weaker `Signature` trait bound (the other bounds are de facto enforced by the generated code).
    type Handle: Signature;
}

impl<'e, T> Signature for T
where
    T: JavaValue<'e>,
{
    const SIG_TYPE: &'static str = T::SIG_TYPE;
}

impl<'e, T: JavaValue<'e>> HandleDispatcher<'e> for T {
    type Handle = T;
}
