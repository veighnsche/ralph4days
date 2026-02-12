use std::env;

fn print_models(agent: &str) {
    println!("{agent}:");
    for model in ralph4days_lib::list_provider_model_entries(Some(agent)) {
        println!("{}: {}", model.name, model.description);
    }
}

fn main() {
    let mode = env::args().nth(1).unwrap_or_else(|| "both".to_owned());
    match mode.as_str() {
        "codex" => print_models("codex"),
        "claude" => print_models("claude"),
        "both" => {
            print_models("codex");
            println!();
            print_models("claude");
        }
        other => {
            eprintln!("Unknown provider '{other}'. Expected one of: codex, claude, both");
            std::process::exit(2);
        }
    }
}
