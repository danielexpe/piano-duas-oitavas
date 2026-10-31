use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn script_available() -> bool {
    Command::new("script")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[test]
fn linux_term_falls_back_to_auto_keyup() {
    if !script_available() {
        eprintln!("script(1) indisponível; pulando teste PTY");
        return;
    }

    let binary = env!("CARGO_BIN_EXE_piano-duas-oitavas");
    let command = format!("stty rows 24 cols 80; exec {binary} --no-audio");
    let mut child = Command::new("script")
        .args(["-q", "-e", "-c", &command, "/dev/null"])
        .env("TERM", "linux")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("falha ao iniciar script(1)");

    let mut stdin = child.stdin.take().expect("stdin do script");
    thread::sleep(Duration::from_millis(120));
    stdin.write_all(b"\x1b").expect("Esc");
    stdin.flush().expect("flush do Esc");
    drop(stdin);

    let output = child.wait_with_output().expect("fim do processo");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "status {:?}", output.status);
    assert!(stdout.contains("Key-up: auto"), "HUD não indicou auto");
    assert!(
        stdout.contains("\u{1b}[?1049l"),
        "alternate screen não restaurado"
    );
}

#[test]
fn tui_restores_the_terminal_on_escape() {
    if !script_available() {
        eprintln!("script(1) indisponível; pulando teste PTY");
        return;
    }

    let binary = env!("CARGO_BIN_EXE_piano-duas-oitavas");
    let command = format!("stty rows 24 cols 80; exec {binary} --no-audio");
    let mut child = Command::new("script")
        .args(["-q", "-e", "-c", &command, "/dev/null"])
        .env("TERM", "xterm-256color")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("falha ao iniciar script(1)");

    let mut stdin = child.stdin.take().expect("stdin do script");
    thread::sleep(Duration::from_millis(200));
    stdin
        .write_all(b"\x1b[?1u\x1b[?1;2c")
        .expect("resposta do protocolo de teclado");
    stdin.flush().expect("flush do protocolo");
    thread::sleep(Duration::from_millis(80));
    stdin.write_all(b"z").expect("nota Z");
    stdin.flush().expect("flush da nota");
    thread::sleep(Duration::from_millis(120));
    stdin.write_all(b"\x1b").expect("Esc");
    stdin.flush().expect("flush do Esc");
    drop(stdin);

    let output = child.wait_with_output().expect("fim do processo");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "status {:?}; stderr: {stderr}",
        output.status
    );
    assert!(
        stdout.contains("\u{1b}[?1049h"),
        "alternate screen não foi ativado"
    );
    assert!(
        stdout.contains("\u{1b}[?1049l"),
        "alternate screen não foi restaurado"
    );
    assert!(
        stdout.contains("Piano Duas Oitavas") || stdout.contains("Oitava"),
        "a TUI não desenhou o piano"
    );
}
