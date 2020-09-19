use jni::JNIEnv;
use jni::objects::{JList, JObject, JString, JValue};
use jni::sys::{jboolean, jbooleanArray, jchar, jobject, jstring};

use crate::convert::JavaValue;

/// Conversion trait from Rust values to Java values, analogous to [`Into`]. Used when converting types returned from JNI-available functions.
pub trait IntoJavaValue<'env> {
    type Target: JavaValue<'env>;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target;
}

/// Conversion trait from Rust values to Java values, analogous to [`From`]. Used when converting types that are input to JNI-available functions.
pub trait FromJavaValue<'env> {
    type Source: JavaValue<'env>;

    fn from(s: Self::Source, env: &JNIEnv<'env>) -> Self;
}

impl<'env, T> IntoJavaValue<'env> for T where T: JavaValue<'env> {
    type Target = T;

    fn into(self, _: &JNIEnv<'env>) -> Self::Target {
        self
    }
}

impl<'env, T> FromJavaValue<'env> for T where T: JavaValue<'env> {
    type Source = T;

    fn from(t: Self::Source, _: &JNIEnv<'env>) -> Self {
        t
    }
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

impl<'env> FromJavaValue<'env> for bool {
    type Source = jboolean;

    fn from(s: Self::Source, _env: &JNIEnv<'env>) -> Self {
        s == 0
    }
}

impl<'env> IntoJavaValue<'env> for char {
    type Target = jchar;

    fn into(self, _env: &JNIEnv<'env>) -> Self::Target {
        self as jchar
    }
}

impl<'env> FromJavaValue<'env> for char {
    type Source = jchar;

    fn from(s: Self::Source, _env: &JNIEnv<'env>) -> Self {
        // TODO: Check validity of this unsafe block
        unsafe {
            std::mem::transmute(s as u32)
        }
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

impl<'env> FromJavaValue<'env> for Box<[bool]> {
    type Source = jbooleanArray;

    fn from(s: Self::Source, env: &JNIEnv<'env>) -> Self {
        let len = env.get_array_length(s).unwrap();
        let mut buf = Vec::with_capacity(len as usize).into_boxed_slice();
        env.get_boolean_array_region(s, 0, &mut *buf).unwrap();

        buf.iter().map(|&b| FromJavaValue::from(b, &env)).collect()
    }
}

impl<'env, T> IntoJavaValue<'env> for Vec<T> where T: IntoJavaValue<'env> {
    type Target = jobject;

    fn into(self, env: &JNIEnv<'env>) -> Self::Target {
        let obj = env.new_object("java/util/ArrayList", "(I)V", &[JValue::Int(self.len() as i32)]).unwrap();
        let list = JList::from_env(&env, obj).unwrap();

        self.into_iter()
            .map(|el| JavaValue::autobox(IntoJavaValue::into(el, &env), &env))
            .for_each(|el| {
                list.add(el).unwrap();
            });

        list.into_inner()
    }
}

impl<'env, T, U> FromJavaValue<'env> for Vec<T> where T: FromJavaValue<'env, Source=U>, U: JavaValue<'env> {
    type Source = JObject<'env>;

    fn from(s: Self::Source, env: &JNIEnv<'env>) -> Self {
        let list = JList::from_env(env, s).unwrap();

        list.iter().unwrap()
            .map(|el| {
                T::from(U::unbox(el, env), env)
            })
            .collect()
    }
}
