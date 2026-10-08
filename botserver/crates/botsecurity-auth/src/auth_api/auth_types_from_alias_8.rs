use super::Permission;

impl Permission {
    pub(crate) fn from_alias_chunk8(s_lower: &str) -> Option<Self> {
    if s_lower == "learncaneditcourses" { return Some(Self::LearnCanEditCourses); }
    if s_lower == "learncandeletecourses" { return Some(Self::LearnCanDeleteCourses); }
    if s_lower == "learncanmanagemodules" { return Some(Self::LearnCanManageModules); }
    if s_lower == "codecanread" { return Some(Self::CodeCanRead); }
    if s_lower == "codecanwrite" { return Some(Self::CodeCanWrite); }
    if s_lower == "codecandelete" { return Some(Self::CodeCanDelete); }
    if s_lower == "codecanexecute" { return Some(Self::CodeCanExecute); }
    if s_lower == "codecanmanagegit" { return Some(Self::CodeCanManageGit); }
    if s_lower == "codecancommit" { return Some(Self::CodeCanCommit); }
    if s_lower == "codecanpush" { return Some(Self::CodeCanPush); }
    if s_lower == "codecandeploy" { return Some(Self::CodeCanDeploy); }
    if s_lower == "databasecanquery" { return Some(Self::DatabaseCanQuery); }
    if s_lower == "databasecanreadtables" { return Some(Self::DatabaseCanReadTables); }
    if s_lower == "databasecanwritetables" { return Some(Self::DatabaseCanWriteTables); }
    if s_lower == "databasecandeletetables" { return Some(Self::DatabaseCanDeleteTables); }
    if s_lower == "databasecanadmintables" { return Some(Self::DatabaseCanAdminTables); }
    if s_lower == "databasecanmanagemigrations" { return Some(Self::DatabaseCanManageMigrations); }
    if s_lower == "templatescanview" { return Some(Self::TemplatesCanView); }
    if s_lower == "templatescancreate" { return Some(Self::TemplatesCanCreate); }
    if s_lower == "templatescanedit" { return Some(Self::TemplatesCanEdit); }
    if s_lower == "templatescandelete" { return Some(Self::TemplatesCanDelete); }
    if s_lower == "templatescanapply" { return Some(Self::TemplatesCanApply); }
    if s_lower == "listscanview" { return Some(Self::ListsCanView); }
    if s_lower == "listscancreate" { return Some(Self::ListsCanCreate); }
    if s_lower == "listscanedit" { return Some(Self::ListsCanEdit); }
    if s_lower == "listscandelete" { return Some(Self::ListsCanDelete); }
    if s_lower == "listscanexport" { return Some(Self::ListsCanExport); }
    if s_lower == "listscanimport" { return Some(Self::ListsCanImport); }
    None
    }
}

