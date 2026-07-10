import test from "node:test";
import assert from "node:assert/strict";
import { runFluxError } from "../helpers/flux-cli.js";

test("un comando desconocido termina con error", () => {
    const result = runFluxError(["unknown"]);

    assert.equal(result.stdout, "");
    assert.match(result.stderr, /comando no reconocido: unknown/);
});

test("el codigo de salida distingue errores de ejecucion", () => {
    const result = runFluxError(["settle"]);

    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /flux-dtl:/);
});
