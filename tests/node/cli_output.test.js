import test from "node:test";
import assert from "node:assert/strict";
import { runFlux } from "../helpers/flux-cli.js";

test("el comando demo imprime el resumen operativo", () => {
    const stdout = runFlux(["demo"]);

    assert.match(stdout, /Flux DTL demo executed/);
    assert.match(stdout, /accounts: 3/);
    assert.match(stdout, /assets: 3/);
    assert.match(stdout, /vaults: 3/);
    assert.match(stdout, /lanes: 2/);
    assert.match(stdout, /epochs: 1/);
    assert.match(stdout, /events: 15/);
    assert.match(stdout, /settled_volume: 355352790548/);
    assert.match(stdout, /liquidity_credits: 146392081/);
});

test("el comando help lista los comandos del binario", () => {
    const stdout = runFlux(["help"]);

    assert.match(stdout, /Flux DTL settlement console/);
    assert.match(stdout, /flux-dtl demo/);
    assert.match(stdout, /flux-dtl demo-json/);
});
