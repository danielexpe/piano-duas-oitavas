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
    assert!(help.contains("--fb-device"));
    assert!(help.contains("--capture-frame"));

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
fn fb_backend_reports_a_missing_device() {
    let output = run(&["--backend", "fb", "--fb-device", "/dev/does-not-exist"]);
    assert_eq!(code(&output), 3);
    assert!(String::from_utf8_lossy(&output.stderr).contains("backend indisponível"));
}

#[test]
fn capture_frame_writes_a_png() {
    let path = std::env::temp_dir().join(format!(
        "piano-frame-{}-{}.png",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    let output = run(&["--capture-frame", &path.display().to_string()]);
    assert_eq!(code(&output), 0);
    let bytes = std::fs::read(&path).expect("capture file");
    let _ = std::fs::remove_file(&path);
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    assert!(bytes.len() > 1000);
}

#[test]
fn tui_without_a_terminal_exits_with_code_three() {
    let output = run(&[]);
    assert_eq!(code(&output), 3);
    assert!(String::from_utf8_lossy(&output.stderr).contains("interactive terminal"));
}
