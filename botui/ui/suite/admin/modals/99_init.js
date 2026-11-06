/* Admin modals — export to window (kept last so every handler exists). */
(function (window) {
  "use strict";
  var A = window.__GBAdminModals;
  var showModal = A.showModal;
  var hideModal = A.hideModal;
  var showNotification = A.showNotification;
  var showSmtpModal = A.showSmtpModal;
  var closeSmtpModal = A.closeSmtpModal;
  var testSmtpConnection = A.testSmtpConnection;
  var connectAccount = A.connectAccount;
  var disconnectAccount = A.disconnectAccount;
  var showInviteMemberModal = A.showInviteMemberModal;
  var closeInviteMemberModal = A.closeInviteMemberModal;
  var showBulkInviteModal = A.showBulkInviteModal;
  var closeBulkInviteModal = A.closeBulkInviteModal;
  var getCurrentOrgId = A.getCurrentOrgId;
  var refreshInvitations = A.refreshInvitations;
  var sendInvitation = A.sendInvitation;
  var sendBulkInvitations = A.sendBulkInvitations;
  var resendInvitation = A.resendInvitation;
  var cancelInvitation = A.cancelInvitation;
  var updateBillingPeriod = A.updateBillingPeriod;
  var exportBillingReport = A.exportBillingReport;
  var toggleBreakdownView = A.toggleBreakdownView;
  var showQuotaSettings = A.showQuotaSettings;
  var closeQuotaSettings = A.closeQuotaSettings;
  var saveQuotaSettings = A.saveQuotaSettings;
  var configureAlerts = A.configureAlerts;
  var closeAlertsConfig = A.closeAlertsConfig;
  var saveAlertSettings = A.saveAlertSettings;
  var showUpgradeModal = A.showUpgradeModal;
  var closeUpgradeModal = A.closeUpgradeModal;
  var showCancelModal = A.showCancelModal;
  var closeCancelModal = A.closeCancelModal;
  var showAddPaymentModal = A.showAddPaymentModal;
  var closeAddPaymentModal = A.closeAddPaymentModal;
  var showEditAddressModal = A.showEditAddressModal;
  var closeEditAddressModal = A.closeEditAddressModal;
  var exportInvoices = A.exportInvoices;
  var contactSales = A.contactSales;
  var showDowngradeOptions = A.showDowngradeOptions;
  var selectPlan = A.selectPlan;
  var confirmUpgrade = A.confirmUpgrade;
  var confirmCancellation = A.confirmCancellation;
  var updateFramework = A.updateFramework;
  var generateComplianceReport = A.generateComplianceReport;
  var startAuditPrep = A.startAuditPrep;
  var closeAuditPrep = A.closeAuditPrep;
  var showEvidenceUpload = A.showEvidenceUpload;
  var closeEvidenceUpload = A.closeEvidenceUpload;
  var uploadEvidence = A.uploadEvidence;
  var filterLogs = A.filterLogs;
  var exportAuditLog = A.exportAuditLog;
  var closeDetailPanel = A.closeDetailPanel;
  var openDetailPanel = A.openDetailPanel;
  var createGroup = A.createGroup;
  var closeCreateGroup = A.closeCreateGroup;
  var saveGroup = A.saveGroup;
  var deleteGroup = A.deleteGroup;
  var selectRole = A.selectRole;
  var renderRoleDetail = A.renderRoleDetail;
  var loadRolePermissions = A.loadRolePermissions;
  var renderPermissionLists = A.renderPermissionLists;
  var togglePermissionSelect = A.togglePermissionSelect;
  var assignSelected = A.assignSelected;
  var assignAll = A.assignAll;
  var removeSelected = A.removeSelected;
  var removeAll = A.removeAll;
  var savePermissions = A.savePermissions;
  var resetPermissions = A.resetPermissions;
  var duplicateRole = A.duplicateRole;
  var confirmDeleteRole = A.confirmDeleteRole;
  var deleteRole = A.deleteRole;
  var assignUsersToRole = A.assignUsersToRole;
  var assignGroupsToRole = A.assignGroupsToRole;
  var filterRoles = A.filterRoles;
  var downloadInvoice = A.downloadInvoice;
  var dismissAlert = A.dismissAlert;
  var viewEvidence = A.viewEvidence;
  var toggleAutoRefresh = A.toggleAutoRefresh;
  var startAutoRefresh = A.startAutoRefresh;
  var stopAutoRefresh = A.stopAutoRefresh;
  var showAlertConfig = A.showAlertConfig;
  var closeAlertConfig = A.closeAlertConfig;
  var showTraceDetail = A.showTraceDetail;
  var refreshHealth = A.refreshHealth;
  var nextStep = A.nextStep;
  var prevStep = A.prevStep;
  var updateStepIndicators = A.updateStepIndicators;
  var skipPayment = A.skipPayment;
  var removeLogo = A.removeLogo;
  var previewLogo = A.previewLogo;
  var editAccount = A.editAccount;
  var syncAccount = A.syncAccount;

// EXPORT TO WINDOW
// =============================================================================

// Accounts
window.showSmtpModal = showSmtpModal;
window.closeSmtpModal = closeSmtpModal;
window.testSmtpConnection = testSmtpConnection;
window.connectAccount = connectAccount;
window.disconnectAccount = disconnectAccount;

// Admin Dashboard
window.showInviteMemberModal = showInviteMemberModal;
window.closeInviteMemberModal = closeInviteMemberModal;
window.showBulkInviteModal = showBulkInviteModal;
window.closeBulkInviteModal = closeBulkInviteModal;
window.sendInvitation = sendInvitation;
window.sendBulkInvitations = sendBulkInvitations;
window.resendInvitation = resendInvitation;
window.cancelInvitation = cancelInvitation;

// Billing Dashboard
window.updateBillingPeriod = updateBillingPeriod;
window.exportBillingReport = exportBillingReport;
window.toggleBreakdownView = toggleBreakdownView;
window.showQuotaSettings = showQuotaSettings;
window.closeQuotaSettings = closeQuotaSettings;
window.saveQuotaSettings = saveQuotaSettings;
window.configureAlerts = configureAlerts;
window.closeAlertsConfig = closeAlertsConfig;
window.saveAlertSettings = saveAlertSettings;

// Billing
window.showUpgradeModal = showUpgradeModal;
window.closeUpgradeModal = closeUpgradeModal;
window.showCancelModal = showCancelModal;
window.closeCancelModal = closeCancelModal;
window.showAddPaymentModal = showAddPaymentModal;
window.closeAddPaymentModal = closeAddPaymentModal;
window.showEditAddressModal = showEditAddressModal;
window.closeEditAddressModal = closeEditAddressModal;
window.exportInvoices = exportInvoices;
window.contactSales = contactSales;
window.showDowngradeOptions = showDowngradeOptions;
window.selectPlan = selectPlan;
window.confirmUpgrade = confirmUpgrade;
window.confirmCancellation = confirmCancellation;

// Compliance Dashboard
window.updateFramework = updateFramework;
window.generateComplianceReport = generateComplianceReport;
window.startAuditPrep = startAuditPrep;
window.closeAuditPrep = closeAuditPrep;
window.showEvidenceUpload = showEvidenceUpload;
window.closeEvidenceUpload = closeEvidenceUpload;
window.uploadEvidence = uploadEvidence;
window.filterLogs = filterLogs;
window.exportAuditLog = exportAuditLog;

// Groups
window.closeDetailPanel = closeDetailPanel;
window.openDetailPanel = openDetailPanel;
window.createGroup = createGroup;
window.closeCreateGroup = closeCreateGroup;
window.saveGroup = saveGroup;
window.deleteGroup = deleteGroup;

// Role Management
window.selectRole = selectRole;
window.assignSelected = assignSelected;
window.assignAll = assignAll;
window.removeSelected = removeSelected;
window.removeAll = removeAll;
window.savePermissions = savePermissions;
window.resetPermissions = resetPermissions;
window.duplicateRole = duplicateRole;
window.confirmDeleteRole = confirmDeleteRole;
window.deleteRole = deleteRole;
window.assignUsersToRole = assignUsersToRole;
window.assignGroupsToRole = assignGroupsToRole;
window.filterRoles = filterRoles;

// Billing Admin
window.downloadInvoice = downloadInvoice;
window.dismissAlert = dismissAlert;
window.viewEvidence = viewEvidence;

// Operations Dashboard
window.toggleAutoRefresh = toggleAutoRefresh;
window.showAlertConfig = showAlertConfig;
window.closeAlertConfig = closeAlertConfig;
window.showTraceDetail = showTraceDetail;
window.refreshHealth = refreshHealth;

// Onboarding
window.nextStep = nextStep;
window.prevStep = prevStep;
window.skipPayment = skipPayment;
window.removeLogo = removeLogo;
window.previewLogo = previewLogo;

// Account Management
window.editAccount = editAccount;
window.syncAccount = syncAccount;
})(window);
