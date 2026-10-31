use std::process::{Command, Output, Stdio};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_piano-duas-oitavas"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("failed to run piano-duas-oitavas")
}

fn code(output: &Output) -> i32 {
    output
        .status
        .code()
        .expect("process was terminated by signal")
}

#[test]
fn help_and_version_succeed() {
    let help = run(&["--help"]);
    assert_eq!(code(&help), 0);
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("--octave"));
    assert!(help.contains("--volume"));
    assert!(help.contains("--backend"));

    let version = run(&["--version"]);
    assert_eq!(code(&version), 0);
}

#[test]
fn invalid_cli_values_exit_with_code_two() {
    assert_eq!(code(&run(&["--octave", "0"])), 2);
    assert_eq!(code(&run(&["--octave", "7"])), 2);
    assert_eq!(code(&run(&["--volume", "-0.1"])), 2);
    assert_eq!(code(&run(&["--volume", "1.1"])), 2);
    assert_eq!(code(&run(&["--lang", "es"])), 2);
    assert_eq!(code(&run(&["--backend", "gpu"])), 2);
}

#[test]
fn fb_backend_is_unavailable_until_phase_six() {
    let output = run(&["--backend", "fb"]);
    assert_eq!(code(&output), 3);
    assert!(String::from_utf8_lossy(&output.stderr).contains("backend indisponível"));
}

#[test]
fn capture_frame_is_not_implemented_yet() {
    let output = run(&["--capture-frame", "/tmp/piano-frame.png"]);
    assert_eq!(code(&output), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("--capture-frame"));
}

#[test]
fn tui_without_a_terminal_exits_with_code_three() {
    let output = run(&[]);
    assert_eq!(code(&output), 3);
    assert!(String::from_utf8_lossy(&output.stderr).contains("interactive terminal"));
}
