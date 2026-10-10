use std::io::{self, Read};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--serve"] {
        if let Err(error) =
            kanata_studio_core::protocol::serve(io::stdin().lock(), io::stdout().lock())
        {
            eprintln!("kanata-studio-helper: {error}");
            std::process::exit(1);
        }
        return;
    }
    if !args.is_empty() {
        eprintln!("Usage: kanata-studio-helper [--serve]");
        std::process::exit(2);
    }
    let mut input = String::new();
    let result = io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())
        .and_then(|_| kanata_studio_core::dispatch(&mut Default::default(), &input));
    match result {
        Ok(value) => println!("{value}"),
        Err(error) => {
            println!("{}", serde_json::json!({"ok":false, "error":error}));
            std::process::exit(1);
        }
    }
}
