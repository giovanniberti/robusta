use jni::{objects::JObject, JNIEnv};

pub struct Local<'a: 'b, 'b> {
    obj: JObject<'a>,
    #[allow(dead_code)]
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b> Local<'a, 'b> {
    pub fn new(env: &'b JNIEnv<'a>, obj: JObject<'a>) -> Self {
        Local { obj, env }
    }

    /// Get a reference to the wrapped object
    pub fn as_obj<'c>(&self) -> JObject<'c>
        where
            'a: 'c,
    {
        self.obj
    }
}

impl<'a, 'b> Drop for Local<'a, 'b> {
    fn drop(&mut self) {}
}

impl<'a> From<&'a Local<'a, '_>> for JObject<'a> {
    fn from(other: &'a Local) -> JObject<'a> {
        other.as_obj()
    }
}
