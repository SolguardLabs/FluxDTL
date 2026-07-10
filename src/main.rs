#![allow(dead_code)]

mod accounts;
mod amount;
mod asset;
mod epochs;
mod error;
mod events;
mod ids;
mod lanes;
mod ledger;
mod oracle;
mod orders;
mod risk;
mod settlement;
mod vault;

use ledger::Ledger;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = args.first().map(String::as_str).unwrap_or("demo");

    let result = match command {
        "demo" => run_demo(false),
        "demo-json" => run_demo(true),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(format!("comando no reconocido: {other}")),
    };

    if let Err(error) = result {
        eprintln!("flux-dtl: {error}");
        std::process::exit(1);
    }
}

fn run_demo(json: bool) -> Result<(), String> {
    let report = Ledger::demo().map_err(|error| error.to_string())?;

    if json {
        println!("{}", report.to_json());
    } else {
        println!("Flux DTL demo executed");
        println!("accounts: {}", report.accounts);
        println!("assets: {}", report.assets);
        println!("vaults: {}", report.vaults);
        println!("lanes: {}", report.lanes);
        println!("epochs: {}", report.epochs);
        println!("events: {}", report.events);
        println!("settled_volume: {}", report.settled_volume);
        println!("liquidity_credits: {}", report.liquidity_credits);
    }

    Ok(())
}

fn print_help() {
    println!("Flux DTL settlement console");
    println!();
    println!("USAGE:");
    println!("  flux-dtl demo");
    println!("  flux-dtl demo-json");
}
