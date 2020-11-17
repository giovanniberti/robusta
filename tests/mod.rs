use std::process::Command;

#[test]
fn java_integration_tests() {
    std::env::set_current_dir("./tests/driver").expect("unable to change directory to ./tests/driver");
    let mut child = Command::new("./gradlew")
        .arg("test")
        .spawn()
        .expect("Failed to execute command");

    let exit_status = child.wait().expect("Failed to wait on gradle");

    assert!(exit_status.success())
}