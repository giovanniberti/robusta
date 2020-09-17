use jni::errors::Result as Result;
use jni::JNIEnv;

use crate::convert::JavaValue;
use crate::convert::unchecked::{FromJavaValue, IntoJavaValue};

/// Conversion trait from Rust values to Java values, analogous to [`std::convert::TryInto`]. Used when converting types returned from JNI-available functions.
pub trait TryIntoJavaValue<'env> where Self: Default {
    type Target: JavaValue<'env>;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target>;
}

/// Conversion trait from Rust values to Java values, analogous to [`std::convert::TryFrom`]. Used when converting types that are input to JNI-available functions.
pub trait TryFromJavaValue<'env> where Self: Sized {
    type Source: JavaValue<'env>;

    fn try_from(s: Self::Source, env: &JNIEnv<'env>) -> Result<Self>;
}

impl<'env, T> TryIntoJavaValue<'env> for T where T: JavaValue<'env> + Default {
    type Target = T;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        Ok(IntoJavaValue::into(self, env))
    }
}

impl<'env, T> TryFromJavaValue<'env> for T where T: JavaValue<'env> {
    type Source = T;

    fn try_from(s: Self::Source, env: &JNIEnv<'env>) -> Result<Self> {
        Ok(FromJavaValue::from(s, env))
    }
}