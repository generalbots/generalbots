/* Drive Module v2.0 — 01 State */
"use strict";

// ── Tab Constants ──────────────────────────────────────────────
const TAB_BRANCHDRIVE = "branchdrive";
const TAB_SHARED      = "shared";
const TAB_PUBLIC      = "public";
const TAB_MYFILES     = "myfiles";
const TAB_BOTS        = "bots";
const TAB_ROOT        = "root";
const TAB_DESKTOP     = "desktop";

// ── State Variables ─────────────────────────────────────────────
const API_BASE = "/api/files";

let currentBucket = "";
let currentPath = "";
let currentScope = sessionStorage.getItem("drive-scope") || "user";
let availableBuckets = [];
let selectedFiles = new Set();
let viewMode = "list";
let clipboardFiles = [];
let clipboardOperation = null;
let retryCount = 0;
const MAX_RETRIES = 3;
const RETRY_DELAYS = [1000, 3000, 10000];

let userInfo = { is_anonymous: true, roles: [] };
let isAdmin = false;
let currentGborgBucket = null;
let currentGborgBranch = null;

// Tab state
let currentTab = "branchdrive";
let sharedCache = [];
