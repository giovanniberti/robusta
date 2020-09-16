use jni::JNIEnv;
use jni::objects::JObject;

pub mod unchecked;
pub mod safe;

/// A trait for types that are ffi-safe to use with JNI. It is implemented for primitives, [`JObject`] and [`jobject`].
/// User that wants automatic conversion should instead implement [`FromJavaValue`] and [`IntoJavaValue`]
pub trait JavaValue<'env> {
    fn autobox(self, env: &JNIEnv<'env>) -> JObject<'env>;

    fn unbox(s: JObject<'env>, env: &JNIEnv<'env>) -> Self;
}
