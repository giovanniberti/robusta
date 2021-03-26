use std::process::Command;
use robusta_jni::jni::{InitArgsBuilder, JavaVM};
use native::jni::User;
use jni::objects::JString;
use robusta_jni::convert::FromJavaValue;

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
    let current_dir = std::env::current_dir().expect("Couldn't get current dir");
    let classpath = current_dir.join("./tests/driver/build/classes/java/main");

    let vm_args  = InitArgsBuilder::new()
        .option(&*format!("-Djava.class.path={}", classpath.to_string_lossy()))
        .build().expect("can't create vm args");
    let vm = JavaVM::new(vm_args).expect("can't create vm");
    let env = vm.attach_current_thread().expect("can't get vm env");

    User::initNative();

    let count = {
        User::getTotalUsersCount(&env)
            .or_else(|e| {
                let ex = env.exception_occurred()?;
                env.exception_clear()?;
                let res = env.call_method(ex, "toString","()Ljava/lang/String;", &[])?;
                let message: JString = From::from(res.l()?);
                let s: String = FromJavaValue::from(message, &env);
                println!("Java exception occurred: {}", s);
                Err(e)
            })
            .expect("can't get user count")
    };

    assert_eq!(count, 0);
}
