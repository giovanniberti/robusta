use robusta_jni::bridge;

#[bridge]
mod jni {
    use android_logger::Config;
    use jni::objects::JObject;
    use log::info;
    use robusta_jni::convert::{Signature, IntoJavaValue, TryIntoJavaValue, TryFromJavaValue};
    use robusta_jni::jni::JNIEnv;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::errors::Result as JniResult;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(com.example.robustaandroidexample)]
    pub struct RobustaAndroidExample<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
    }

    impl<'env: 'borrow, 'borrow> RobustaAndroidExample<'env, 'borrow> {

        pub extern "jni" fn runRustExample(self, env: &JNIEnv, context: JObject<'env>) {
            android_logger::init_once(
                Config::default()
                    .with_min_level(log::Level::Debug)
                    .with_tag("RUST_ROBUSTA_ANDROID_EXAMPLE"),
            );
           let app_files_dir = RobustaAndroidExample::getAppFilesDir(env, context).unwrap();
            info!("App files dir: {}", app_files_dir);
        }

        pub extern "java" fn getAppFilesDir(
            env: &JNIEnv,
            #[input_type("Landroid/content/Context;")] context: JObject
        ) -> JniResult<String> {}
    }
}
