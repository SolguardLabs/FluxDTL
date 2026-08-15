import test from "node:test";
import assert from "node:assert/strict";
import { FluxClient } from "../../sdk/client.js";
import { normalizeReport, operationalSnapshot } from "../../sdk/report.js";

const deterministicReport = {
    accounts: 3,
    assets: 3,
    vaults: 3,
    lanes: 2,
    epochs: 1,
    events: 15,
    settled_volume: 355352790548,
    liquidity_credits: 146392081,
};

test("client returns a validated deterministic report", () => {
    const client = new FluxClient();
    assert.deepEqual(client.demo(), deterministicReport);
});

test("client exposes an operational snapshot", () => {
    const snapshot = new FluxClient().snapshot();
    assert.equal(snapshot.status, "operational");
    assert.equal(snapshot.creditRatioBps, 4);
    assert.equal(snapshot.topology, "3 assets / 2 lanes / 3 vaults");
});

test("client validates timeout configuration", () => {
    assert.throws(() => new FluxClient({ timeoutMs: 0 }), /positive integer/);
});

test("client propagates a structured command failure", () => {
    const client = new FluxClient();
    assert.throws(
        () => client.run(["unsupported-command"]),
        (error) => error.exitCode !== 0 && /comando no reconocido/.test(error.stderr),
    );
});

test("report normalization rejects incomplete input", () => {
    assert.throws(() => normalizeReport({ accounts: 1 }), /field assets/);
});

test("snapshot handles an idle zero-volume report", () => {
    const report = Object.fromEntries(Object.keys(deterministicReport).map((key) => [key, 0]));
    const snapshot = operationalSnapshot(report);
    assert.equal(snapshot.status, "idle");
    assert.equal(snapshot.creditRatioBps, 0);
    assert.equal(snapshot.settlementPerEvent, 0);
});

test("stress command returns a conservative treasury assessment", () => {
    const report = JSON.parse(new FluxClient().run(["stress-json"]));
    assert.equal(report.assets, 3);
    assert.equal(report.band, "healthy");
    assert.ok(report.coverage_bps > 11_500);
    assert.equal(report.low_confidence_assets, 0);
});
