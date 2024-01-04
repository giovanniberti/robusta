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
    let mut child = Command::new("./gradlew")
        .args(&["test", "-i"])
        .current_dir("./tests/driver")
        .spawn()
        .expect("Failed to execute command");

    let exit_status = child.wait().expect("Failed to wait on gradle");

    assert!(exit_status.success())
}

#[test]
fn vm_creation_and_object_usage() {
    let mut child = Command::new("./gradlew")
        .args(&["test", "-i"])
        .current_dir("./tests/driver")
        .spawn()
        .expect("Failed to execute command");

    let exit_status = child.wait().expect("Failed to wait on gradle build");
    assert!(exit_status.success());

    let current_dir = std::env::current_dir().expect("Couldn't get current dir");
    let classpath = current_dir.join("./tests/driver/build/classes/java/main");

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

    assert_eq!(User::getNullableString(&env, None).expect("can't get nullable string"), None);
    assert_eq!(User::getNullableString(&env, Some("hello!".into())).expect("can't get nullable string"), Some("hello!".into()));
    assert_eq!(User::getNullableStringUnchecked(&env, None), None);
    assert_eq!(User::getNullableStringUnchecked(&env, Some("hello!".into())), Some("hello!".into()));

    let count = User::getTotalUsersCount(&env)
        .or_else(|e| {
            let _ = print_exception(&env);
            Err(e)
        })
        .expect("can't get user count");

    assert_eq!(count, 0);
    assert_eq!(User::getTotalUsersCountUnchecked(&env), 0);

    let u = User::new(&env, "user".into(), "password".into()).expect("can't create user instance");

    let count = User::getTotalUsersCount(&env)
        .or_else(|e| {
            let _ = print_exception(&env);
            Err(e)
        })
        .expect("can't get user count");
    assert_eq!(count, 1);
    assert_eq!(User::getTotalUsersCountUnchecked(&env), 1);

    assert_eq!(
        u.getPassword(&env).expect("can't get user password"),
        "password"
    );

    assert_eq!(
        u.getPasswordUnchecked(&env),
        "password"
    );

    assert_eq!(
        u.multipleParameters(&env, 10, "test".to_string())
            .expect("Can't test multipleParameters"),
        "test"
    );

    assert_eq!(
        u.multipleParametersUnchecked(&env, 10, "test".to_string()),
        "test"
    );
}
