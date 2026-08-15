import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const currentDir = dirname(fileURLToPath(import.meta.url));
export const projectRoot = resolve(currentDir, "../..");

export function runFlux(args = []) {
    const result = spawnSync("cargo", ["run", "--quiet", "--", ...args], {
        cwd: projectRoot,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
        timeout: 30_000,
        maxBuffer: 4 * 1024 * 1024,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stderr);
    return result.stdout;
}

export function runFluxError(args = []) {
    const result = spawnSync("cargo", ["run", "--quiet", "--", ...args], {
        cwd: projectRoot,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
        timeout: 30_000,
        maxBuffer: 4 * 1024 * 1024,
    });

    assert.equal(result.error, undefined);
    assert.notEqual(result.status, 0);
    return result;
}

export function loadDemoReport() {
    const report = JSON.parse(runFlux(["demo-json"]));

    assert.equal(typeof report, "object");
    assert.ok(report);
    return report;
}
