use crate::jni::RobustaAndroidExample;
use jni::objects::JValue;
use log::{debug, error};

pub(crate) fn thread_test_fail() -> Result<(), String> {
    debug!("TEST_THREAD_FAIL: start...");

    let (app_vm, _) = crate::APP_CONTEXT
        .get()
        .ok_or_else(|| "Couldn't get APP_CONTEXT".to_string())?;
    let env = app_vm
        .attach_current_thread_permanently()
        .map_err(|_| "Couldn't attach to current thread".to_string())?;

    debug!("TEST_THREAD_FAIL: via JNI");
    let test_string = env.new_string("SUPER TEST").unwrap();
    let test_string = JValue::from(test_string);
    if let Err(e) = env.call_static_method(
        "com/example/robustaandroidexample/RobustaAndroidExample",
        "threadTestNoClass",
        "(Ljava/lang/String;)I",
        &[test_string],
    ) {
        error!("Couldn't call method via classic JNI: {}", e);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
    }

    debug!("TEST_THREAD_FAIL: via Robusta");

    /* Call methode */
    if let Err(e) = RobustaAndroidExample::threadTestNoClass(&env, "test".to_string()) {
        let msg = format!("Couldn't call method via Robusta: {}", e);
        error!("{}", msg);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
        return Err(msg);
    }
    Ok(())
}

pub(crate) fn thread_test_good() -> Result<(), String> {
    debug!("TEST_THREAD_GOOD: start...");

    let (app_vm, class_ref) = crate::APP_CONTEXT
        .get()
        .ok_or_else(|| "Couldn't get APP_CONTEXT".to_string())?;
    let env = app_vm
        .attach_current_thread_permanently()
        .map_err(|_| "Couldn't attach to current thread".to_string())?;

    debug!("TEST_THREAD_GOOD: via JNI");
    let test_string = env.new_string("SUPER TEST").unwrap();
    let test_string = JValue::from(test_string);
    if let Err(e) = env.call_static_method(
        class_ref,
        "threadTestNoClass",
        "(Ljava/lang/String;)I",
        &[test_string],
    ) {
        error!("Couldn't call method via classic JNI: {}", e);
        if env.exception_check().unwrap_or(false) {
            let ex = env.exception_occurred().unwrap();
            let _ = env.exception_clear();
            let res = env
                .call_method(ex, "toString", "()Ljava/lang/String;", &[])
                .unwrap()
                .l()
                .unwrap();
            let ex_msg: String = env.get_string(res.into()).unwrap().into();
            error!("check_jni_error: {}", ex_msg);
        }
    }

    debug!("TEST_THREAD_GOOD: via Robusta");

    /* Call methode */
    if let Err(e) = RobustaAndroidExample::threadTestWithClass(&env, class_ref, "test".to_string())
    {
        let msg = format!("Couldn't call method via Robusta: {}", e);
        error!("{}", msg);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
        return Err(msg);
    }
    Ok(())
}
