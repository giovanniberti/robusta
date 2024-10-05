pub(crate) mod thread_func;

use ::jni::objects::GlobalRef;
use ::jni::JavaVM;
use robusta_jni::bridge;
use std::sync::OnceLock;

static APP_CONTEXT: OnceLock<(JavaVM, GlobalRef)> = OnceLock::new();

#[bridge]
mod jni {
    use crate::APP_CONTEXT;
    use android_logger::Config;
    use jni::objects::{GlobalRef, JObject, JValue};
    use log::info;
    use robusta_jni::convert::{IntoJavaValue, Signature, TryFromJavaValue, TryIntoJavaValue};
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::JNIEnv;
    use std::thread;
    use robusta_jni::convert::Local;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(com.example.robustaandroidexample)]
    pub struct RobustaAndroidExample<'env: 'borrow, 'borrow> {
        #[instance]
        raw: Local<'env, 'borrow>,
    }

    impl<'env: 'borrow, 'borrow> RobustaAndroidExample<'env, 'borrow> {
        pub extern "jni" fn runRustExample(self, env: &JNIEnv, context: JObject<'env>) {
            android_logger::init_once(Config::default().with_tag("RUST_ROBUSTA_ANDROID_EXAMPLE"));

            info!("TEST START");
            let java_class = env
                .find_class("com/example/robustaandroidexample/RobustaAndroidExample")
                .unwrap();
            let _ = APP_CONTEXT.set((
                env.get_java_vm().unwrap(),
                env.new_global_ref(java_class).unwrap(),
            ));

            let app_files_dir = RobustaAndroidExample::getAppFilesDir(env, context).unwrap();
            info!("App files dir: {}", app_files_dir);

            assert_eq!(
                RobustaAndroidExample::threadTestNoClass(env, "test".to_string()).unwrap(),
                10
            );

            let test_string = env.new_string("SUPER TEST").unwrap();
            let test_string = JValue::from(test_string);
            let met_call = env.call_static_method(
                "com/example/robustaandroidexample/RobustaAndroidExample",
                "threadTestNoClass",
                "(Ljava/lang/String;)I",
                &[test_string],
            );
            assert!(met_call.is_ok());

            let thread_handler = thread::Builder::new()
                .name("test_thread_fail".to_string())
                .spawn(move || crate::thread_func::thread_test_fail());
            let join_res = thread_handler.unwrap().join().unwrap();
            assert!(join_res.is_err());

            let thread_handler = thread::Builder::new()
                .name("test_thread_good".to_string())
                .spawn(move || crate::thread_func::thread_test_good());
            let join_res = thread_handler.unwrap().join().unwrap();
            assert!(join_res.is_ok());

            info!("TEST END");
        }

        pub extern "java" fn getAppFilesDir(
            env: &JNIEnv,
            #[input_type("Landroid/content/Context;")] context: JObject,
        ) -> JniResult<String> {
        }

        pub extern "java" fn threadTestNoClass(env: &JNIEnv, s: String) -> JniResult<i32> {}
        pub extern "java" fn threadTestWithClass(
            env: &JNIEnv,
            class_ref: &GlobalRef,
            s: String,
        ) -> JniResult<i32> {
        }
    }
}
