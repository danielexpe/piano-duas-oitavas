fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("assets") => {
            println!("xtask assets: implemente o pipeline determinístico em xtask/src/main.rs");
            if args.iter().any(|arg| arg == "--check") {
                println!("xtask assets --check: nada a verificar ainda");
            }
            Ok(())
        }
        _ => {
            eprintln!("uso: cargo xtask assets [--check]");
            std::process::exit(2);
        }
    }
}
