use super::Permission;

impl Permission {
    pub(crate) fn from_alias_chunk6(s_lower: &str) -> Option<Self> {
    if s_lower == "automationcancreateworkflows" { return Some(Self::AutomationCanCreateWorkflows); }
    if s_lower == "automationcaneditworkflows" { return Some(Self::AutomationCanEditWorkflows); }
    if s_lower == "automationcandeleteworkflows" { return Some(Self::AutomationCanDeleteWorkflows); }
    if s_lower == "automationcanexecuteworkflows" { return Some(Self::AutomationCanExecuteWorkflows); }
    if s_lower == "automationcanmanagetriggers" { return Some(Self::AutomationCanManageTriggers); }
    if s_lower == "automationcanmanageschedules" { return Some(Self::AutomationCanManageSchedules); }
    if s_lower == "automationcanmanageeventhandlers" { return Some(Self::AutomationCanManageEventHandlers); }
    if s_lower == "crmcanviewpipeline" { return Some(Self::CrmCanViewPipeline); }
    if s_lower == "crmcanmanageleads" { return Some(Self::CrmCanManageLeads); }
    if s_lower == "crmcanmanagecontacts" { return Some(Self::CrmCanManageContacts); }
    if s_lower == "crmcanmanagedeals" { return Some(Self::CrmCanManageDeals); }
    if s_lower == "crmcanviewreports" { return Some(Self::CrmCanViewReports); }
    if s_lower == "crmcanexportreports" { return Some(Self::CrmCanExportReports); }
    if s_lower == "crmcanmanageforecast" { return Some(Self::CrmCanManageForecast); }
    if s_lower == "campaignscancreate" { return Some(Self::CampaignsCanCreate); }
    if s_lower == "campaignscanedit" { return Some(Self::CampaignsCanEdit); }
    if s_lower == "campaignscandelete" { return Some(Self::CampaignsCanDelete); }
    if s_lower == "campaignscanexecute" { return Some(Self::CampaignsCanExecute); }
    if s_lower == "campaignscanviewanalytics" { return Some(Self::CampaignsCanViewAnalytics); }
    if s_lower == "campaignscanmanagesegments" { return Some(Self::CampaignsCanManageSegments); }
    if s_lower == "productscanviewcatalog" { return Some(Self::ProductsCanViewCatalog); }
    if s_lower == "productscancreateproducts" { return Some(Self::ProductsCanCreateProducts); }
    if s_lower == "productscaneditproducts" { return Some(Self::ProductsCanEditProducts); }
    if s_lower == "productscandeleteproducts" { return Some(Self::ProductsCanDeleteProducts); }
    if s_lower == "productscancreateservices" { return Some(Self::ProductsCanCreateServices); }
    if s_lower == "productscaneditservices" { return Some(Self::ProductsCanEditServices); }
    if s_lower == "productscandeleteservices" { return Some(Self::ProductsCanDeleteServices); }
    if s_lower == "productscanmanagepricelists" { return Some(Self::ProductsCanManagePriceLists); }
    if s_lower == "ticketscancreate" { return Some(Self::TicketsCanCreate); }
    if s_lower == "ticketscanread" { return Some(Self::TicketsCanRead); }
    if s_lower == "ticketscanupdate" { return Some(Self::TicketsCanUpdate); }
    if s_lower == "ticketscandelete" { return Some(Self::TicketsCanDelete); }
    if s_lower == "ticketscanassign" { return Some(Self::TicketsCanAssign); }
    if s_lower == "ticketscanresolve" { return Some(Self::TicketsCanResolve); }
    if s_lower == "ticketscanmanagepriorities" { return Some(Self::TicketsCanManagePriorities); }
    if s_lower == "ticketscanviewanalytics" { return Some(Self::TicketsCanViewAnalytics); }
    if s_lower == "ticketscanmanageattendant" { return Some(Self::TicketsCanManageAttendant); }
    if s_lower == "peoplecanviewdirectory" { return Some(Self::PeopleCanViewDirectory); }
    if s_lower == "peoplecanmanagecontacts" { return Some(Self::PeopleCanManageContacts); }
    if s_lower == "peoplecanmanagegroups" { return Some(Self::PeopleCanManageGroups); }
    if s_lower == "peoplecanmanageroles" { return Some(Self::PeopleCanManageRoles); }
    if s_lower == "peoplecanimportcontacts" { return Some(Self::PeopleCanImportContacts); }
    if s_lower == "browsercannavigate" { return Some(Self::BrowserCanNavigate); }
    if s_lower == "browsercanbookmark" { return Some(Self::BrowserCanBookmark); }
    if s_lower == "browsercanmanagehistory" { return Some(Self::BrowserCanManageHistory); }
    if s_lower == "browsercandownload" { return Some(Self::BrowserCanDownload); }
    if s_lower == "terminalcanexecute" { return Some(Self::TerminalCanExecute); }
    if s_lower == "terminalcanviewoutput" { return Some(Self::TerminalCanViewOutput); }
    if s_lower == "terminalcanmanagesessions" { return Some(Self::TerminalCanManageSessions); }
    if s_lower == "researchcansearch" { return Some(Self::ResearchCanSearch); }
    if s_lower == "researchcanmanagesources" { return Some(Self::ResearchCanManageSources); }
    if s_lower == "researchcanexportresults" { return Some(Self::ResearchCanExportResults); }
    if s_lower == "researchcanmanagesessions" { return Some(Self::ResearchCanManageSessions); }
    if s_lower == "socialcanpost" { return Some(Self::SocialCanPost); }
    if s_lower == "socialcanscheduleposts" { return Some(Self::SocialCanSchedulePosts); }
    None
    }
}

