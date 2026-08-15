const REPORT_FIELDS = [
    "accounts",
    "assets",
    "vaults",
    "lanes",
    "epochs",
    "events",
    "settled_volume",
    "liquidity_credits",
];

export function normalizeReport(value) {
    if (value === null || typeof value !== "object" || Array.isArray(value)) {
        throw new TypeError("FluxDTL report must be an object");
    }

    const report = {};
    for (const field of REPORT_FIELDS) {
        const candidate = value[field];
        if (!Number.isSafeInteger(candidate) || candidate < 0) {
            throw new TypeError(`FluxDTL report field ${field} must be a non-negative integer`);
        }
        report[field] = candidate;
    }
    return Object.freeze(report);
}

export function operationalSnapshot(value) {
    const report = normalizeReport(value);
    const settlementPerEvent = report.events === 0 ? 0 : report.settled_volume / report.events;
    const creditRatioBps =
        report.settled_volume === 0
            ? 0
            : Math.floor((report.liquidity_credits * 10_000) / report.settled_volume);

    return Object.freeze({
        report,
        settlementPerEvent,
        creditRatioBps,
        topology: `${report.assets} assets / ${report.lanes} lanes / ${report.vaults} vaults`,
        status: report.epochs > 0 && report.events > 0 ? "operational" : "idle",
    });
}
