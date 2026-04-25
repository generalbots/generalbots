function selectDeploymentTarget(target) {
selectedDeploymentTarget = target;

var internalOption = document.getElementById("deploymentInternal");
var externalOption = document.getElementById("deploymentExternal");
var internalConfig = document.getElementById(
"deploymentInternalConfig",
);
var externalConfig = document.getElementById(
"deploymentExternalConfig",
);

if (target === "internal") {
internalOption.style.borderColor = "var(--accent)";
internalOption.style.background = "rgba(132, 214, 105, 0.06)";
externalOption.style.borderColor = "var(--border)";
externalOption.style.background = "transparent";
internalConfig.style.display = "block";
externalConfig.style.display = "none";
} else {
externalOption.style.borderColor = "var(--accent)";
externalOption.style.background = "rgba(132, 214, 105, 0.06)";
internalOption.style.borderColor = "var(--border)";
internalOption.style.background = "transparent";
internalConfig.style.display = "none";
externalConfig.style.display = "block";
}
}

function showDeploymentModal() {
var modal = document.getElementById("vibeDeploymentModal");
if (modal) {
modal.style.display = "block";
selectDeploymentTarget("internal");
}
}

function closeDeploymentModal() {
var modal = document.getElementById("vibeDeploymentModal");
if (modal) {
modal.style.display = "none";
}
}

async function executeDeployment() {
var deployButton = document.getElementById("deployButton");
if (deployButton) {
deployButton.textContent = "Deploying...";
deployButton.disabled = true;
}

let payload = {
app_name: document.getElementById('deployRepoName')?.value || document.getElementById('deployRoute')?.value || "my-app",
target: {},
environment: "production",
manifest: {}
};

if (selectedDeploymentTarget === 'external') {
payload.target = {
External: {
repo_url: "https://alm.pragmatismo.com.br/" + payload.app_name,
custom_domain: document.getElementById('deployCustomDomain')?.value || null,
ci_cd_enabled: document.getElementById('deployCiCd')?.checked ?? true
}
};
payload.app_type = document.getElementById('deployAppType')?.value || "htmx";
} else {
let route = document.getElementById('deployRoute')?.value || "my-app";
payload.target = {
Internal: {
route: "/apps/" + route,
shared_resources: document.getElementById('deploySharedResources')?.checked ?? true
}
};
payload.app_type = "gb-native";
}

try {
vibeAddMsg("system", "🚀 Initiating deployment API call...");
const response = await fetch('/api/deployment/deploy', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify(payload)
});

const result = await response.json();

if (response.ok && (result.success || result.status === 'Deployed' || result.status === 'Building')) {
closeDeploymentModal();
vibeAddMsg("system", "✅ Deployment Successful! " + (result.url ? result.url : ""));

var previewUrl = document.getElementById("vibePreviewUrl");
var previewPanel = document.getElementById("vibePreview");
if (previewUrl && previewPanel && result.url) {
previewUrl.value = result.url;
previewPanel.style.display = "block";
}
} else {
vibeAddMsg("system", "❌ Deployment failed: " + (result.error || result.status));
}
} catch (e) {
vibeAddMsg("system", "❌ Deployment error: " + e.message);
} finally {
if (deployButton) {
deployButton.textContent = "Deploy Now";
deployButton.disabled = false;
}
}
}

if (document.readyState !== "loading") {
var routeInput = document.getElementById("deployRoute");
if (routeInput) {
routeInput.addEventListener("input", function () {
var preview = document.getElementById("deployRoutePreview");
if (preview) {
preview.textContent = this.value || "my-app";
}
});
}
}
