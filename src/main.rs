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
mod treasury;
mod vault;

use amount::Amount;
use ids::AssetId;
use ledger::Ledger;
use treasury::{AssetExposure, StressPolicy, TreasuryStressEngine};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = args.first().map(String::as_str).unwrap_or("demo");

    let result = match command {
        "demo" => run_demo(false),
        "demo-json" => run_demo(true),
        "stress-json" => run_stress(),
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

fn run_stress() -> Result<(), String> {
    let exposures = [
        AssetExposure {
            asset: AssetId::new(1),
            reserve: Amount::new(240_000_000_000),
            committed: Amount::new(90_000_000_000),
            expected_inflow: Amount::new(20_000_000_000),
            price_e8: 100_000_000,
            confidence_bps: 9_980,
            haircut_bps: 300,
        },
        AssetExposure {
            asset: AssetId::new(2),
            reserve: Amount::new(180_000_000_000),
            committed: Amount::new(100_000_000_000),
            expected_inflow: Amount::new(12_000_000_000),
            price_e8: 108_000_000,
            confidence_bps: 9_970,
            haircut_bps: 500,
        },
        AssetExposure {
            asset: AssetId::new(3),
            reserve: Amount::new(4_000_000_000_000),
            committed: Amount::new(2_000_000_000_000),
            expected_inflow: Amount::new(300_000_000_000),
            price_e8: 5_800_000,
            confidence_bps: 9_940,
            haircut_bps: 1_200,
        },
    ];
    let report = TreasuryStressEngine::assess(StressPolicy::default(), &exposures)
        .map_err(|error| error.to_string())?;
    println!(
        "{{\"band\":\"{}\",\"assets\":{},\"gross_reserve_value_e8\":{},\"stressed_resources_value_e8\":{},\"committed_value_e8\":{},\"liquidity_gap_value_e8\":{},\"coverage_bps\":{},\"largest_concentration_bps\":{},\"low_confidence_assets\":{}}}",
        report.band,
        report.assets.len(),
        report.gross_reserve_value_e8,
        report.stressed_resources_value_e8,
        report.committed_value_e8,
        report.liquidity_gap_value_e8,
        report.coverage_bps,
        report.largest_concentration_bps,
        report.low_confidence_assets,
    );
    Ok(())
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
    println!("  flux-dtl stress-json");
}
