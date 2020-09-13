use jni::objects::{JString};
use jni::sys::jstring;
use jni::JNIEnv;

pub trait JavaValue {}

pub trait IntoJavaValue<'env> {
    type Target: JavaValue;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target;
}

pub trait FromJavaValue<'env> {
    type Source: JavaValue;

    fn from(s: Self::Source, env: &JNIEnv<'env>) -> Self;
}


// TODO: Find out wether is possible to write a blanket impl IntoJavaValue for T where T: FromJavaValue as std does
impl<'env, T> IntoJavaValue<'env> for T where T: JavaValue {
    type Target = T;

    fn into(self, _: &JNIEnv<'env>) -> Self::Target {
        self
    }
}

impl<'env, T> FromJavaValue<'env> for T where T: JavaValue {
    type Source = T;

    fn from(t: Self::Source, _: &JNIEnv<'env>) -> Self {
        t
    }
}

macro_rules! jvalue_types {
    ($type:ty) => {
        impl JavaValue for $type {}
    };

    ($type:ty, $($rest:ty),+) => {
        jvalue_types!($type);

        jvalue_types!($($rest),+);
    }
}

jvalue_types!{
    JString<'_>,
    jstring
    // TODO: Mark other types
}

impl<'env> IntoJavaValue<'env> for String {
    type Target = jstring;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        env.new_string(self).unwrap().into_inner()
    }
}

impl<'env> FromJavaValue<'env> for String {
    type Source = JString<'env>;

    fn from(s: Self::Source, env: &JNIEnv<'env>) -> Self {
        env.get_string(s).unwrap().into()
    }
}