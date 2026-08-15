import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const testDirectory = join(root, "tests", "node");
const files = readdirSync(testDirectory)
    .filter((name) => name.endsWith(".test.js"))
    .sort()
    .map((name) => join(testDirectory, name));

if (files.length === 0) {
    throw new Error("No Node test files were found");
}

const result = spawnSync(process.execPath, ["--test", ...files], {
    cwd: root,
    encoding: "utf8",
    stdio: "inherit",
    timeout: 120_000,
});

if (result.error) {
    throw result.error;
}
process.exit(result.status ?? 1);
