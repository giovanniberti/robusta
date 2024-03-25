use std::fs;
use std::path::Path;
use jni::objects::{JObject, JString};
use native::jni::User;
use robusta_jni::convert::{FromJavaValue, JavaValue};
use robusta_jni::jni::{InitArgsBuilder, JNIEnv, JavaVM};
use std::process::Command;
use native::StringArr;

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

    assert_eq!(User::nullableString(&env, None).expect("can't get nullable string"), None);
    assert_eq!(User::nullableString(&env, Some("hello!".into())).expect("can't get nullable string"), Some("hello!".into()));
    assert_eq!(User::nullableStringUnchecked(&env, None), None);
    assert_eq!(User::nullableStringUnchecked(&env, Some("hello!".into())), Some("hello!".into()));

    assert_eq!(User::nullableDouble(&env, None).expect("can't get nullable double"), 0f64);
    assert_eq!(User::nullableDouble(&env, Some(4.2f64)).expect("can't get nullable double"), 4.2f64);
    assert_eq!(User::nullableDoubleUnchecked(&env, None), 0f64);
    assert_eq!(User::nullableDoubleUnchecked(&env, Some(4.2f64)), 4.2f64);

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
                                vec![Some("42".to_string()), None],
                                vec![42, 42].into_boxed_slice(), vec![false, true].into_boxed_slice(),
                                vec![env.new_string("42").unwrap(), <JString<'_> as From<JObject>>::from(JObject::null())].into_boxed_slice(),
                                vec!["42".to_string(), "42".to_string()].into_boxed_slice(),
                                vec![None, Some("42".to_string())].into_boxed_slice(),
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
        "[42, null]",
        "[42, 42]", "[false, true]",
        "[42, null]",
        "[42, 42]",
        "[null, 42]",
        "null", "[[42], null]",
        "[[42], [42, 42]]",
        "[[42], null]",
        "[[42], [42, 42]]",
        "[[42], null]",
        "[[42]]",
    ]);

    assert_eq!(
        User::signaturesCheckUnchecked(
            &env,
            42, false, '2', 42, 42.0, 42.0, 42, 42, "42".to_string(),
            vec![42, 42, 42], vec!["42".to_string(), "42".to_string()],
            vec![Some("42".to_string()), None],
            vec![42, 42].into_boxed_slice(), vec![false, true].into_boxed_slice(),
            vec![env.new_string("42").unwrap(), <JString<'_> as From<JObject>>::from(JObject::null())].into_boxed_slice(),
            vec!["42".to_string(), "42".to_string()].into_boxed_slice(),
            vec![None, Some("42".to_string())].into_boxed_slice(),
            None, vec![Some(vec![42].into_boxed_slice()), None],
            vec![vec![42].into_boxed_slice(), vec![42, 42].into_boxed_slice()],
            vec![Some(vec!["42".to_string()].into_boxed_slice()), None],
            vec![vec!["42".to_string()].into_boxed_slice(), vec!["42".to_string(), "42".to_string()].into_boxed_slice()],
            vec![Some(Into::into(vec!["42".to_string()].into_boxed_slice())), None].into_boxed_slice(),
            vec![Into::into(vec!["42".to_string()].into_boxed_slice())].into_boxed_slice(),
        ), vec![
            "42", "false", "2", "42", "42.0", "42.0", "42", "42", "42",
            "[42, 42, 42]", "[42, 42]",
            "[42, null]",
            "[42, 42]", "[false, true]",
            "[42, null]",
            "[42, 42]",
            "[null, 42]",
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
    let borrow_user = create_user("borrow", "42");
    let borrow_user_opt = create_user("borrow_opt", "42");
    let res = u.selfSignatureCheck(
        &env,
        create_user("user", "42"),
        &borrow_user, Some(&borrow_user_opt), Some(&borrow_user_opt),
        Some(create_user("user", "null")), None,
        vec![create_user("user", "pass")],
        vec![Some(create_user("user", "null")), None],
        Some(vec![create_user("user", "arr_null")]), None,
        vec![create_user("login", "42")].into_boxed_slice(),
        vec![Some(create_user("user", "null")), None].into_boxed_slice(),
        Some(vec![create_user("login", "arr_null")].into_boxed_slice()), None,
    ).expect("can't check self signature");
    assert_eq!(res, vec![
        "User{username='user', password='password'}",
        "User{username='user', password='42'}",
        "User{username='user', password='null'}", "null",
        "[User{username='user', password='pass'}]",
        "[User{username='user', password='null'}, null]",
        "[User{username='user', password='arr_null'}]", "null",
        "[User{username='login', password='42'}]",
        "[User{username='user', password='null'}, null]",
        "[User{username='login', password='arr_null'}]", "null",
    ]);

    assert_eq!(
        u.selfSignatureCheckUnchecked(
            &env,
            create_user("user", "42"),
            &borrow_user, Some(&borrow_user_opt), Some(&borrow_user_opt),
            Some(create_user("user", "null")), None,
            vec![create_user("user", "pass")],
            vec![Some(create_user("user", "null")), None],
            Some(vec![create_user("user", "arr_null")]), None,
            vec![create_user("login", "42")].into_boxed_slice(),
            vec![Some(create_user("user", "null")), None].into_boxed_slice(),
            Some(vec![create_user("login", "arr_null")].into_boxed_slice()), None,
        ), vec![
            "User{username='user', password='password'}",
            "User{username='user', password='42'}",
            "User{username='user', password='null'}", "null",
            "[User{username='user', password='pass'}]",
            "[User{username='user', password='null'}, null]",
            "[User{username='user', password='arr_null'}]", "null",
            "[User{username='login', password='42'}]",
            "[User{username='user', password='null'}, null]",
            "[User{username='login', password='arr_null'}]", "null",
        ]
    );
    // Mutable data fields can be tricky, as data is copied only once
    // when (Try)FromJavaValue is called
    assert_eq!("42", borrow_user.password);
    assert_eq!("42__", borrow_user.getPassword(&env).expect("unable to get password"));
    // We actually reinitialize data field here
    assert_eq!("42__", User::cloneUser(&env, &borrow_user).password);
    // Mutable class fields work as expected
    assert_eq!("borrow__", borrow_user.username.get().expect("unable to get username"));
    // This field is simply not updated on the java side
    let expected = "42".as_bytes();
    assert_eq!(
        unsafe {
            &*std::ptr::slice_from_raw_parts(expected.as_ref().as_ptr() as *const i8, expected.as_ref().len())
        }.to_vec(),
        borrow_user.bytes.get().expect("unable to get bytes").to_vec(),
    );

    assert_eq!(borrow_user.toString(&env), "User{username='borrow__', password='42__'}");
    assert_eq!(borrow_user_opt.toString(&env), "User{username='borrow_opt____', password='42____'}");

    let mut res = User::stringArrNullable2D(
        &env,
        Some(Into::into(vec![
            "42".to_string()
        ].into_boxed_slice())),
        None,
    ).expect("can't check 2D string array").into_vec();
    assert_eq!(res.len(), 2);
    assert!(res[0].is_none());
    let res1 = res.remove(1);
    assert!(res1.is_some());
    assert_eq!(
        <StringArr as Into<Box<[String]>>>::into(res1.unwrap()),
        vec!["42".to_string()].into_boxed_slice(),
    );

    let mut res = u.stringArrNullable2DUnchecked(
        &env,
        Some(Into::into(vec![
            "42".to_string()
        ].into_boxed_slice())),
        None,
    ).into_vec();
    assert_eq!(res.len(), 2);
    assert!(res[0].is_none());
    let res1 = res.remove(1);
    assert!(res1.is_some());
    assert_eq!(
        <StringArr as Into<Box<[String]>>>::into(res1.unwrap()),
        vec!["42".to_string()].into_boxed_slice(),
    );

    assert_eq!(
        <f64 as JavaValue>::unbox(
            u.typeOverrideJava(
                &env, <f64 as JavaValue>::autobox(4.2f64, &env)
            ).expect("unable to call typeOverrideJava"), &env
        ), -4.2f64);
    assert_eq!(
        <f64 as JavaValue>::unbox(
            User::typeOverrideJavaUnchecked(
                &env, <f64 as JavaValue>::autobox(4.2f64, &env)
            ), &env
        ), -4.2f64);
}
