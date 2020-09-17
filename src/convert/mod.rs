use jni::JNIEnv;
use jni::objects::JObject;

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
