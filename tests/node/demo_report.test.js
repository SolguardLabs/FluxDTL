import test from "node:test";
import assert from "node:assert/strict";
import { loadDemoReport } from "../helpers/flux-cli.js";

test("demo-json devuelve metricas deterministas", () => {
    const report = loadDemoReport();

    assert.deepEqual(report, {
        accounts: 3,
        assets: 3,
        vaults: 3,
        lanes: 2,
        epochs: 1,
        events: 15,
        settled_volume: 355352790548,
        liquidity_credits: 146392081,
    });
});

test("las metricas del reporte mantienen relaciones esperadas", () => {
    const report = loadDemoReport();

    assert.equal(report.accounts, 3);
    assert.equal(report.assets, report.vaults);
    assert.ok(report.settled_volume > 300000000000);
    assert.ok(report.liquidity_credits > 0);
    assert.ok(report.settled_volume > report.liquidity_credits);
});
