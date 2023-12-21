use jni::objects::JString;
use native::jni::User;
use robusta_jni::convert::FromJavaValue;
use robusta_jni::jni::{InitArgsBuilder, JNIEnv, JavaVM};
use std::process::Command;

fn print_exception(env: &JNIEnv) -> jni::errors::Result<()> {
    let ex = env.exception_occurred()?;
    env.exception_clear()?;
    let res = env.call_method(ex, "toString", "()Ljava/lang/String;", &[])?;
    let message: JString = From::from(res.l()?);
    let s: String = FromJavaValue::from(message, env);
    println!("Java exception occurred: {}", s);
    Ok(())
}

#[test]
fn java_integration_tests() {
    let mut child = if cfg!(target_os = "windows") {
        Command::new("gradlew.bat")
    } else {
        Command::new("./gradlew")
    }.args(&["test", "-i"])
        .current_dir(
            std::path::Path::new(".").join("tests").join("driver").to_str().expect("Failed to get driver path")
        )
        .spawn()
        .expect("Failed to execute command");

    let exit_status = child.wait().expect("Failed to wait on gradle");

    assert!(exit_status.success())
}

#[test]
fn vm_creation_and_object_usage() {
    let mut child = if cfg!(target_os = "windows") {
        Command::new("gradlew.bat")
    } else {
        Command::new("./gradlew")
    }.args(&["test", "-i"])
        .current_dir(
            std::path::Path::new(".").join("tests").join("driver").to_str().expect("Failed to get driver path")
        )
        .spawn()
        .expect("Failed to execute command");

    let exit_status = child.wait().expect("Failed to wait on gradle build");
    assert!(exit_status.success());

    let current_dir = std::env::current_dir().expect("Couldn't get current dir");
    let classpath = current_dir.join("tests").join("driver").join("build").join("classes").join("java").join("main");

    // Cargo sets DYLD_FALLBACK_LIBRARY_PATH on os x, but java uses DYLD_LIBRARY_PATH to set java.library.path
    std::env::set_var(
        "DYLD_LIBRARY_PATH",
        format!(
            "{}:{}",
            std::env::var("DYLD_LIBRARY_PATH").unwrap_or("".to_string()),
            std::env::var("DYLD_FALLBACK_LIBRARY_PATH").unwrap_or("".to_string()),
        ));
    let vm_args = InitArgsBuilder::new()
        .option(&*format!(
            "-Djava.class.path={}",
            classpath.to_string_lossy()
        ))
        .build()
        .expect("can't create vm args");
    let vm = JavaVM::new(vm_args).expect("can't create vm");
    let env = vm.attach_current_thread().expect("can't get vm env");

    User::initNative();

    let count = User::getTotalUsersCount(&env)
        .or_else(|e| {
            let _ = print_exception(&env);
            Err(e)
        })
        .expect("can't get user count");

    assert_eq!(count, 0);

    let u = User::new(&env, "user".into(), "password".into()).expect("can't create user instance");

    let count = User::getTotalUsersCount(&env)
        .or_else(|e| {
            let _ = print_exception(&env);
            Err(e)
        })
        .expect("can't get user count");
    assert_eq!(count, 1);

    assert_eq!(
        u.getPassword(&env).expect("can't get user password"),
        "password"
    );

    assert_eq!(
        u.multipleParameters(&env, 10, "test".to_string())
            .expect("Can't test multipleParameters"),
        "test"
    )
}
