use std::fs;
use std::path::Path;
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
    let mut child = Command::new(
        fs::canonicalize(
            Path::new(".").join("tests").join("driver").join(
                if cfg!(target_os = "windows") { "gradlew.bat" } else { "gradlew" })
        ).expect("Gradle not found"))
        .args(&["test", "-i"])
        .current_dir(
            Path::new(".").join("tests").join("driver").to_str().expect("Failed to get driver path")
        )
        .spawn()
        .expect("Failed to execute command");

    let exit_status = child.wait().expect("Failed to wait on gradle");

    assert!(exit_status.success())
}

#[test]
fn vm_creation_and_object_usage() {
    let mut child = Command::new(
        fs::canonicalize(
            Path::new(".").join("tests").join("driver").join(
                if cfg!(target_os = "windows") { "gradlew.bat" } else { "gradlew" })
        ).expect("Gradle not found"))
        .args(&["compileTestJava"])
        .current_dir(
            Path::new(".").join("tests").join("driver").to_str().expect("Failed to get driver path")
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
    let u_unchecked = User::newUnchecked(&env, "user".into());

    let count = User::getTotalUsersCount(&env)
        .or_else(|e| {
            let _ = print_exception(&env);
            Err(e)
        })
        .expect("can't get user count");
    assert_eq!(count, 2);
    assert_eq!(User::getTotalUsersCountUnchecked(&env), 2);

    assert_eq!(
        u.getPassword(&env).expect("can't get user password"),
        "password"
    );

    assert_eq!(
        u.getPasswordUnchecked(&env),
        "password"
    );

    assert_eq!(u_unchecked.toString(&env), "User{username='user', password='user_pass'}");

    assert_eq!(
        u.multipleParameters(&env, 10, "test".to_string())
            .expect("Can't test multipleParameters"),
        "test"
    );

    assert_eq!(
        u.multipleParametersUnchecked(&env, 10, "test".to_string()),
        "test"
    );

    let res = u.signaturesCheck(&env,
                                42, false, '2', 42, 42.0, 42.0, 42, 42, "42".to_string(),
                                vec![42, 42, 42], vec!["42".to_string(), "42".to_string()],
                                vec![42, 42].into_boxed_slice(), vec![false, true].into_boxed_slice(),
                                vec![env.new_string("42").unwrap(), env.new_string("42").unwrap()].into_boxed_slice(),
                                vec!["42".to_string(), "42".to_string()].into_boxed_slice(),
                                None, vec![Some(vec![42].into_boxed_slice()), None],
                                vec![vec![42].into_boxed_slice(), vec![42, 42].into_boxed_slice()],
                                vec![Some(vec!["42".to_string()].into_boxed_slice()), None],
                                vec![vec!["42".to_string()].into_boxed_slice(), vec!["42".to_string(), "42".to_string()].into_boxed_slice()],
                                vec![Some(Into::into(vec!["42".to_string()].into_boxed_slice())), None].into_boxed_slice(),
                                vec![Into::into(vec!["42".to_string()].into_boxed_slice())].into_boxed_slice(),
    ).or_else(|e| {
        let _ = print_exception(&env);
        Err(e)
    }).expect("can't check signatures");
    assert_eq!(res, vec![
        "42", "false", "2", "42", "42.0", "42.0", "42", "42", "42",
        "[42, 42, 42]", "[42, 42]",
        "[42, 42]", "[false, true]",
        "[42, 42]",
        "[42, 42]",
        "null", "[[42], null]",
        "[[42], [42, 42]]",
        "[[42], null]",
        "[[42], [42, 42]]",
        "[[42], null]",
        "[[42]]",
    ]);

    assert_eq!(
        User::signaturesCheckUnchecked(&env,
                          42, false, '2', 42, 42.0, 42.0, 42, 42, "42".to_string(),
                          vec![42, 42, 42], vec!["42".to_string(), "42".to_string()],
                          vec![42, 42].into_boxed_slice(), vec![false, true].into_boxed_slice(),
                          vec![env.new_string("42").unwrap(), env.new_string("42").unwrap()].into_boxed_slice(),
                          vec!["42".to_string(), "42".to_string()].into_boxed_slice(),
                          None, vec![Some(vec![42].into_boxed_slice()), None],
                          vec![vec![42].into_boxed_slice(), vec![42, 42].into_boxed_slice()],
                          vec![Some(vec!["42".to_string()].into_boxed_slice()), None],
                          vec![vec!["42".to_string()].into_boxed_slice(), vec!["42".to_string(), "42".to_string()].into_boxed_slice()],
                          vec![Some(Into::into(vec!["42".to_string()].into_boxed_slice())), None].into_boxed_slice(),
                          vec![Into::into(vec!["42".to_string()].into_boxed_slice())].into_boxed_slice(),
        ), vec![
            "42", "false", "2", "42", "42.0", "42.0", "42", "42", "42",
            "[42, 42, 42]", "[42, 42]",
            "[42, 42]", "[false, true]",
            "[42, 42]",
            "[42, 42]",
            "null", "[[42], null]",
            "[[42], [42, 42]]",
            "[[42], null]",
            "[[42], [42, 42]]",
            "[[42], null]",
            "[[42]]",
        ]);

    let create_user = |login: &str, password: &str| -> User {
        User::new(&env, login.into(), password.into()).expect("can't create user instance")
    };
    // // sudo sysctl -w kernel.yama.ptrace_scope=0
    // let url = format!("vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}", std::process::id());
    // std::process::Command::new("code").arg("--open-url").arg(url).output().unwrap();
    // std::thread::sleep_ms(10000);
    let res = u.selfSignatureCheck(&env,
                                   create_user("user", "42"),
                                   vec![create_user("user", "pass")],
                                   vec![create_user("login", "42")].into_boxed_slice(),
    ).expect("can't check self signature");
    assert_eq!(res, vec![
        "User{username='user', password='password'}",
        "User{username='user', password='42'}",
        "[User{username='user', password='pass'}]",
        "[User{username='login', password='42'}]",
    ]);

    assert_eq!(
        u.selfSignatureCheckUnchecked(
            &env,
            create_user("user", "42"),
            vec![create_user("user", "pass")],
            vec![create_user("login", "42")].into_boxed_slice(),
        ), vec![
            "User{username='user', password='password'}",
            "User{username='user', password='42'}",
            "[User{username='user', password='pass'}]",
            "[User{username='login', password='42'}]",
        ]
    );
}
