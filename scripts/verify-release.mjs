import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const protectedFiles = new Map([
    ["src/ledger/state.rs", "DCC595D37EABBE5B2C707E6FD94C57BCD10B536D0C7A28A20B62A992C64B0F9E"],
    [
        "src/settlement/engine.rs",
        "4723BDD778BBCC513414D6E035739EB16C04F0599047F29FBA9145A5324707B7",
    ],
    ["src/lanes/config.rs", "A5963BDDDDBBECAF811858E4231ADFF8C053EA8CC637A31E30334C1974336AF8"],
    ["src/vault/state.rs", "BDAE7C4496A558E1E24F20ACB2B58AEDB691CC10DAA6107A3BA01E47023EA5CE"],
]);
const bannerHash = "A56E5DFF5AB1FA92786615A00DB36C7E2293138D0DA1C1C1EBF487CED51A68A8";

function sha256(path) {
    return createHash("sha256")
        .update(readFileSync(join(root, path)))
        .digest("hex")
        .toUpperCase();
}

function collect(directory) {
    const entries = [];
    for (const name of readdirSync(join(root, directory))) {
        if ([".git", "node_modules", "target", "private"].includes(name)) continue;
        const path = join(directory, name);
        const details = statSync(join(root, path));
        entries.push(...(details.isDirectory() ? collect(path) : [path]));
    }
    return entries;
}

for (const [path, expected] of protectedFiles) {
    assert.equal(sha256(path), expected, `${path} changed from the reviewed economic baseline`);
}
assert.equal(sha256("assets/banner.png"), bannerHash, "release banner changed unexpectedly");

const cargo = readFileSync(join(root, "Cargo.toml"), "utf8");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
assert.match(cargo, /^version = "1\.0\.0"$/m);
assert.equal(pkg.version, "1.0.0");

const docs = readdirSync(join(root, "docs")).filter((name) => name.endsWith(".md"));
assert.equal(docs.length, 7, "docs/ must contain the seven operational guides");
const markdown = ["README.md", "SECURITY.md", ...docs.map((name) => join("docs", name))];
const mermaidCount = markdown.reduce((total, path) => {
    return total + (readFileSync(join(root, path), "utf8").match(/```mermaid/g) ?? []).length;
}, 0);
assert.equal(mermaidCount, 27, "the architecture set must contain 27 Mermaid diagrams");

const publicExtensions = new Set([
    ".js",
    ".json",
    ".md",
    ".mjs",
    ".rs",
    ".sh",
    ".toml",
    ".yaml",
    ".yml",
]);
const restrictedWords = [
    "c" + "tf",
    "la" + "b",
    "la" + "bs",
    "labor" + "atorio",
    "labor" + "atorios",
    "vulnera" + "bilidad",
    "vulnera" + "bilidades",
    "vulnera" + "ble",
    "bu" + "g",
    "bu" + "gs",
    "explo" + "it",
    "explo" + "itar",
    "by" + "pass",
    "att" + "acker",
    "atac" + "ante",
];
const restricted = new RegExp(`\\b(?:${restrictedWords.join("|")})\\b`, "iu");
for (const path of collect(".")) {
    if (!publicExtensions.has(extname(path))) continue;
    const content = readFileSync(join(root, path), "utf8");
    assert.equal(
        restricted.test(content),
        false,
        `restricted public wording found in ${relative(root, path)}`,
    );
}

const rustFiles = collect("src").filter((path) => path.endsWith(".rs"));
const rustLoc = rustFiles.reduce(
    (total, path) => total + readFileSync(join(root, path), "utf8").split(/\r?\n/).length,
    0,
);
const rustTests = collect("src")
    .concat(collect("tests"))
    .filter((path) => path.endsWith(".rs"))
    .reduce(
        (total, path) =>
            total + (readFileSync(join(root, path), "utf8").match(/#\[test\]/g) ?? []).length,
        0,
    );
const nodeTests = collect("tests/node")
    .filter((path) => path.endsWith(".test.js"))
    .reduce(
        (total, path) =>
            total + (readFileSync(join(root, path), "utf8").match(/^test\(/gm) ?? []).length,
        0,
    );
assert.ok(rustLoc >= 1_500, `expected at least 1500 Rust lines, received ${rustLoc}`);
assert.ok(rustTests >= 16, `expected at least 16 Rust tests, received ${rustTests}`);
assert.ok(nodeTests >= 12, `expected at least 12 Node tests, received ${nodeTests}`);

console.log(
    `Release verificada: ${rustLoc} lineas Rust, ${rustTests} tests Rust, ${nodeTests} tests Node, ${mermaidCount} diagramas.`,
);
