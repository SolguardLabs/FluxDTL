use serde_json::Value;
use std::process::Command;

fn run_flux(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_flux-dtl"))
        .args(args)
        .output()
        .expect("flux-dtl binary should execute");

    assert!(
        output.status.success(),
        "flux-dtl failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout should be utf8")
}

fn run_flux_error(args: &[&str]) -> (String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_flux-dtl"))
        .args(args)
        .output()
        .expect("flux-dtl binary should execute");

    assert!(
        !output.status.success(),
        "flux-dtl should fail for invalid input"
    );

    (
        String::from_utf8(output.stdout).expect("stdout should be utf8"),
        String::from_utf8(output.stderr).expect("stderr should be utf8"),
    )
}

fn demo_report() -> Value {
    let stdout = run_flux(&["demo-json"]);
    serde_json::from_str(&stdout).expect("demo-json should be valid json")
}

#[test]
fn demo_texto_publica_metricas_operativas() {
    let stdout = run_flux(&["demo"]);

    assert!(stdout.contains("Flux DTL demo executed"));
    assert!(stdout.contains("accounts: 3"));
    assert!(stdout.contains("assets: 3"));
    assert!(stdout.contains("vaults: 3"));
    assert!(stdout.contains("lanes: 2"));
    assert!(stdout.contains("epochs: 1"));
    assert!(stdout.contains("events: 15"));
    assert!(stdout.contains("settled_volume: 355352790548"));
    assert!(stdout.contains("liquidity_credits: 146392081"));
}

#[test]
fn demo_json_contiene_resumen_determinista() {
    let report = demo_report();

    assert_eq!(report["accounts"], 3);
    assert_eq!(report["assets"], 3);
    assert_eq!(report["vaults"], 3);
    assert_eq!(report["lanes"], 2);
    assert_eq!(report["epochs"], 1);
    assert_eq!(report["events"], 15);
    assert_eq!(report["settled_volume"], 355_352_790_548u64);
    assert_eq!(report["liquidity_credits"], 146_392_081u64);
}

#[test]
fn demo_json_mantiene_relaciones_de_contabilidad_basicas() {
    let report = demo_report();
    let settled_volume = report["settled_volume"].as_u64().unwrap();
    let liquidity_credits = report["liquidity_credits"].as_u64().unwrap();

    assert!(settled_volume > 300_000_000_000);
    assert!(liquidity_credits > 0);
    assert!(settled_volume > liquidity_credits);
}

#[test]
fn help_lista_comandos_disponibles() {
    let stdout = run_flux(&["help"]);

    assert!(stdout.contains("Flux DTL settlement console"));
    assert!(stdout.contains("flux-dtl demo"));
    assert!(stdout.contains("flux-dtl demo-json"));
}

#[test]
fn comando_desconocido_falla_con_mensaje_claro() {
    let (stdout, stderr) = run_flux_error(&["unknown"]);

    assert!(stdout.is_empty());
    assert!(stderr.contains("comando no reconocido: unknown"));
}
