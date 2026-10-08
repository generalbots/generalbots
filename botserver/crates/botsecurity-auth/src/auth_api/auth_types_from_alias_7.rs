use super::Permission;

impl Permission {
    pub(crate) fn from_alias_chunk7(s_lower: &str) -> Option<Self> {
    if s_lower == "socialcanviewfeed" { return Some(Self::SocialCanViewFeed); }
    if s_lower == "socialcanmanageaccounts" { return Some(Self::SocialCanManageAccounts); }
    if s_lower == "socialcanviewanalytics" { return Some(Self::SocialCanViewAnalytics); }
    if s_lower == "videocanupload" { return Some(Self::VideoCanUpload); }
    if s_lower == "videocanplay" { return Some(Self::VideoCanPlay); }
    if s_lower == "videocanedit" { return Some(Self::VideoCanEdit); }
    if s_lower == "videocandelete" { return Some(Self::VideoCanDelete); }
    if s_lower == "videocanmanagelibrary" { return Some(Self::VideoCanManageLibrary); }
    if s_lower == "canvascancreate" { return Some(Self::CanvasCanCreate); }
    if s_lower == "canvascanedit" { return Some(Self::CanvasCanEdit); }
    if s_lower == "canvascanview" { return Some(Self::CanvasCanView); }
    if s_lower == "canvascandelete" { return Some(Self::CanvasCanDelete); }
    if s_lower == "canvascanexport" { return Some(Self::CanvasCanExport); }
    if s_lower == "workspacecancreatesites" { return Some(Self::WorkspaceCanCreateSites); }
    if s_lower == "workspacecaneditsites" { return Some(Self::WorkspaceCanEditSites); }
    if s_lower == "workspacecandeletesites" { return Some(Self::WorkspaceCanDeleteSites); }
    if s_lower == "workspacecanviewsites" { return Some(Self::WorkspaceCanViewSites); }
    if s_lower == "workspacecanmanagepages" { return Some(Self::WorkspaceCanManagePages); }
    if s_lower == "workspacecanmanagedatabases" { return Some(Self::WorkspaceCanManageDatabases); }
    if s_lower == "goalscancreate" { return Some(Self::GoalsCanCreate); }
    if s_lower == "goalscanedit" { return Some(Self::GoalsCanEdit); }
    if s_lower == "goalscandelete" { return Some(Self::GoalsCanDelete); }
    if s_lower == "goalscanview" { return Some(Self::GoalsCanView); }
    if s_lower == "goalscantrackprogress" { return Some(Self::GoalsCanTrackProgress); }
    if s_lower == "learncanviewcourses" { return Some(Self::LearnCanViewCourses); }
    if s_lower == "learncanenroll" { return Some(Self::LearnCanEnroll); }
    if s_lower == "learncancreatecourses" { return Some(Self::LearnCanCreateCourses); }
    None
    }
}

