"use strict";

/**
 * Document analytics schema (#2507). Shared by the template, model, renderer,
 * CSV writer and controller modules. API contract for the backend:
 *
 *   GET /api/analytics/documents/summary
 *       ?from=<ISO>&to=<ISO>&range=<24h|7d|30d|90d>
 *       &tenant=<id>&workspace=<id>&department=<id>
 *
 *   {
 *     "generatedAt": "2026-11-19T10:00:00Z",
 *     "scopes": { "tenants": [], "workspaces": [], "departments": [] },
 *     "sections": {
 *       "adoption": {
 *         "activeUsers": [{ "app": "Docs", "workspace": "Sales", "count": 12 }],
 *         "docActivity": [{ "activity": "created", "count": 12 }],
 *         "featureUsage": [{ "feature": "review", "count": 5 }]
 *       },
 *       "process": {
 *         "reviewCycle": [{ "period": "2026-Q3", "p50": 4, "p90": 30 }],
 *         "approvalSla": [{ "period": "2026-Q3", "compliance": 92, "pending": 3 }],
 *         "pendingAgeing": [{ "bucket": "8-30d", "count": 4 }],
 *         "rejectionRate": [{ "period": "2026-Q3", "rate": 2.1 }]
 *       },
 *       "risk": {
 *         "exportsByClassification": [{ "classification": "Confidential", "count": 3 }],
 *         "externalShares": [{ "classification": "Public", "count": 1 }],
 *         "accessAnomalies": [{ "status": "anomaly", "count": 1 }],
 *         "retention": [{ "status": "past-retention", "count": 2 }]
 *       },
 *       "content": {
 *         "storageByType": [{ "type": "document", "bytes": 1048576 }],
 *         "storageByWorkspace": [{ "workspace": "Sales", "bytes": 1048576 }],
 *         "storageByOwner": [{ "ownerBand": "1-5", "bytes": 1048576 }],
 *         "duplicates": [{ "scope": "Sales", "count": 4 }],
 *         "staleDocuments": [{ "bucket": "180d+", "count": 5 }],
 *         "templateReuse": [{ "template": "contract", "count": 9 }]
 *       }
 *     }
 *   }
 *
 * The backend owns aggregation, role scoping and the freshness timestamp.
 * This module never invents values: missing or malformed sections render the
 * explicit "no data available" state.
 */
(function () {
    var ns = (window.AnalyticsDocs = window.AnalyticsDocs || {});

    ns.ENDPOINT = "/api/analytics/documents/summary";
    ns.SUPPRESSION_THRESHOLD = 5;
    ns.FRESHNESS_MAX_MS = 15 * 60 * 1000;
    ns.MAX_ROWS = 100;

    ns.RANGES = { "1h": 1, "6h": 6, "24h": 24, "7d": 168, "30d": 720, "90d": 2160 };

    function tr(key, fallback, params) {
        return typeof window.gbT === "function"
            ? window.gbT(key, fallback, params)
            : fallback;
    }
    ns.tr = tr;

    function col(key, labelKey, type, sensitive) {
        return {
            key: key,
            labelKey: labelKey,
            type: type || "text",
            sensitive: !!sensitive,
        };
    }

    var COUNT = "analytics-docs-col-count";

    ns.SECTIONS = [
        {
            id: "adoption",
            labelKey: "analytics-docs-group-adoption",
            tables: [
                {
                    id: "activeUsers",
                    captionKey: "analytics-docs-caption-active-users",
                    columns: [
                        col("app", "analytics-docs-col-app"),
                        col("workspace", "analytics-docs-workspace"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "docActivity",
                    captionKey: "analytics-docs-caption-doc-activity",
                    columns: [
                        col("activity", "analytics-docs-col-activity"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "featureUsage",
                    captionKey: "analytics-docs-caption-feature-usage",
                    columns: [
                        col("feature", "analytics-docs-col-feature"),
                        col("count", COUNT, "count", true),
                    ],
                },
            ],
        },
        {
            id: "process",
            labelKey: "analytics-docs-group-process",
            tables: [
                {
                    id: "reviewCycle",
                    captionKey: "analytics-docs-caption-review-cycle",
                    columns: [
                        col("period", "analytics-docs-col-period"),
                        col("p50", "analytics-docs-col-p50", "hours"),
                        col("p90", "analytics-docs-col-p90", "hours"),
                    ],
                },
                {
                    id: "approvalSla",
                    captionKey: "analytics-docs-caption-approval-sla",
                    columns: [
                        col("period", "analytics-docs-col-period"),
                        col("compliance", "analytics-docs-col-compliance", "percent", true),
                        col("pending", "analytics-docs-col-pending", "count", true),
                    ],
                },
                {
                    id: "pendingAgeing",
                    captionKey: "analytics-docs-caption-pending-ageing",
                    columns: [
                        col("bucket", "analytics-docs-col-age"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "rejectionRate",
                    captionKey: "analytics-docs-caption-rejection-rate",
                    columns: [
                        col("period", "analytics-docs-col-period"),
                        col("rate", "analytics-docs-col-rate", "percent", true),
                    ],
                },
            ],
        },
        {
            id: "risk",
            labelKey: "analytics-docs-group-risk",
            tables: [
                {
                    id: "exportsByClassification",
                    captionKey: "analytics-docs-caption-exports",
                    columns: [
                        col("classification", "analytics-docs-col-classification"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "externalShares",
                    captionKey: "analytics-docs-caption-external-shares",
                    columns: [
                        col("classification", "analytics-docs-col-classification"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "accessAnomalies",
                    captionKey: "analytics-docs-caption-access-anomalies",
                    columns: [
                        col("status", "analytics-docs-col-status"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "retention",
                    captionKey: "analytics-docs-caption-retention",
                    columns: [
                        col("status", "analytics-docs-col-status"),
                        col("count", COUNT, "count", true),
                    ],
                },
            ],
        },
        {
            id: "content",
            labelKey: "analytics-docs-group-content",
            tables: [
                {
                    id: "storageByType",
                    captionKey: "analytics-docs-caption-storage-type",
                    columns: [
                        col("type", "analytics-docs-col-type"),
                        col("bytes", "analytics-docs-col-bytes", "bytes"),
                    ],
                },
                {
                    id: "storageByWorkspace",
                    captionKey: "analytics-docs-caption-storage-workspace",
                    columns: [
                        col("workspace", "analytics-docs-workspace"),
                        col("bytes", "analytics-docs-col-bytes", "bytes"),
                    ],
                },
                {
                    id: "storageByOwner",
                    captionKey: "analytics-docs-caption-storage-owner",
                    columns: [
                        col("ownerBand", "analytics-docs-col-owner"),
                        col("bytes", "analytics-docs-col-bytes", "bytes"),
                    ],
                },
                {
                    id: "duplicates",
                    captionKey: "analytics-docs-caption-duplicates",
                    columns: [
                        col("scope", "analytics-docs-col-scope"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "staleDocuments",
                    captionKey: "analytics-docs-caption-stale",
                    columns: [
                        col("bucket", "analytics-docs-col-age"),
                        col("count", COUNT, "count", true),
                    ],
                },
                {
                    id: "templateReuse",
                    captionKey: "analytics-docs-caption-templates",
                    columns: [
                        col("template", "analytics-docs-col-template"),
                        col("count", COUNT, "count", true),
                    ],
                },
            ],
        },
    ];

    ns.SCOPE_FIELDS = [
        { id: "docsTenant", name: "tenants", labelKey: "analytics-docs-tenant", param: "tenant" },
        { id: "docsWorkspace", name: "workspaces", labelKey: "analytics-docs-workspace", param: "workspace" },
        { id: "docsDepartment", name: "departments", labelKey: "analytics-docs-department", param: "department" },
    ];

    var EMAIL_RE = /^[^@\s]+@[^@\s]+\.[^@\s]+$/;
    var DOCUMENT_RE = /\.(docx?|xlsx?|pptx?|pdf|odt|ods|odp)$/i;

    /**
     * Dimension labels are categories, never document titles or identities.
     * Anything that looks like an e-mail or a file name is replaced before it
     * reaches the DOM or the CSV.
     */
    ns.safeLabel = function (value) {
        var text = String(value == null ? "" : value).trim();
        if (!text) return tr("analytics-docs-value-none", "Not specified");
        if (EMAIL_RE.test(text) || DOCUMENT_RE.test(text) || text.length > 64) {
            return tr("analytics-docs-value-other", "Other");
        }
        return text;
    };

    var ENUM_LABELS = {
        created: ["analytics-docs-value-created", "Created"],
        edited: ["analytics-docs-value-edited", "Edited"],
        shared: ["analytics-docs-value-shared", "Shared"],
        review: ["analytics-docs-value-review", "Review"],
        comments: ["analytics-docs-value-comments", "Comments"],
        present: ["analytics-docs-value-present", "Present"],
        export: ["analytics-docs-value-export", "Export"],
        "past-retention": ["analytics-docs-value-past-retention", "Past retention"],
        "legal-hold": ["analytics-docs-value-legal-hold", "Legal hold"],
        anomaly: ["analytics-docs-value-anomaly", "Anomaly"],
    };

    ns.valueLabel = function (value) {
        var key = String(value == null ? "" : value);
        var entry = ENUM_LABELS[key];
        return entry ? tr(entry[0], entry[1]) : ns.safeLabel(value);
    };
})();
