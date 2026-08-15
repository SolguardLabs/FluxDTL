import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { normalizeReport, operationalSnapshot } from "./report.js";

const sdkRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));

export class FluxClient {
    constructor({ cwd = sdkRoot, cargo = "cargo", timeoutMs = 30_000, env = {} } = {}) {
        if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) {
            throw new TypeError("timeoutMs must be a positive integer");
        }
        this.cwd = resolve(cwd);
        this.cargo = cargo;
        this.timeoutMs = timeoutMs;
        this.env = { ...process.env, CARGO_TERM_COLOR: "never", ...env };
    }

    run(command) {
        if (!Array.isArray(command) || command.some((item) => typeof item !== "string")) {
            throw new TypeError("command must be an array of strings");
        }
        const result = spawnSync(this.cargo, ["run", "--quiet", "--", ...command], {
            cwd: this.cwd,
            env: this.env,
            encoding: "utf8",
            stdio: ["ignore", "pipe", "pipe"],
            timeout: this.timeoutMs,
            maxBuffer: 4 * 1024 * 1024,
            windowsHide: true,
        });

        if (result.error) {
            throw result.error;
        }
        if (result.status !== 0) {
            const error = new Error(result.stderr.trim() || `FluxDTL exited with ${result.status}`);
            error.exitCode = result.status;
            error.stdout = result.stdout;
            error.stderr = result.stderr;
            throw error;
        }
        return result.stdout;
    }

    demo() {
        return normalizeReport(JSON.parse(this.run(["demo-json"])));
    }

    snapshot() {
        return operationalSnapshot(this.demo());
    }

    help() {
        return this.run(["help"]);
    }
}
