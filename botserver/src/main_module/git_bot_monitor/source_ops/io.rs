use super::*;

/// Read a source file from the bot's repository. `None` when the file (or the
/// `.gbdialog` directory) does not exist yet.
pub(crate) fn read_source(
    pool: &DbPool,
    bot_name: &str,
    name: &str,
) -> Result<Option<String>, String> {
    let target = find_source_target(pool, bot_name)?;
    let checkout = ensure_checkout_with_heal(pool, &target.project, &target.org)?;
    let dialog = match dialog_source_dir(&checkout, bot_name) {
        Some(dir) => dir,
        None => return Ok(None),
    };
    let file = flatten(name)?;
    match std::fs::read_to_string(dialog.join(&file)) {
        Ok(body) => Ok(Some(body)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("read {file}: {e}")),
    }
}

/// List the bot's `.gbdialog` sources plus the project that owns them.
/// A bot with no git project yields `project_id: None` (and no files), which
/// the caller reports instead of guessing a path.
pub(crate) fn list_sources(
    pool: &DbPool,
    bot_name: &str,
) -> Result<botautotask::types::SourceListing, String> {
    let target = find_source_target(pool, bot_name)?;
    let checkout = ensure_checkout_with_heal(pool, &target.project, &target.org)?;
    let dialog = match dialog_source_dir(&checkout, bot_name) {
        Some(dir) => dir,
        None => {
            return Ok(botautotask::types::SourceListing {
                project_id: Some(target.project.project_id),
                files: Vec::new(),
            })
        }
    };
    let mut files: Vec<botautotask::types::SourceFile> = Vec::new();
    for entry in
        std::fs::read_dir(&dialog).map_err(|e| format!("read {}: {e}", dialog.display()))?
    {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        files.push(botautotask::types::SourceFile { name, size });
    }
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(botautotask::types::SourceListing {
        project_id: Some(target.project.project_id),
        files,
    })
}

/// Bot name for a bot id, used to resolve the git target from AutoTask.
fn bot_name_for_id(pool: &DbPool, bot_id: uuid::Uuid) -> Result<String, String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let mut conn = pool.get().map_err(|e| format!("pool: {e}"))?;
    diesel::sql_query("SELECT name FROM bots WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<Row>(&mut conn)
        .map(|r| r.name)
        .map_err(|e| format!("bot {bot_id}: {e}"))
}

/// `botautotask::types::BotSourceOps` adapter over this module, so the AutoTask
/// crate can persist generated sources without depending on botserver.
pub struct GitBotSourceOps {
    pool: DbPool,
}

impl GitBotSourceOps {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl botautotask::types::BotSourceOps for GitBotSourceOps {
    fn write_sources(
        &self,
        bot_id: uuid::Uuid,
        files: &[(String, String)],
        message: &str,
    ) -> Result<Vec<String>, botautotask::types::BoxError> {
        let name = bot_name_for_id(&self.pool, bot_id)?;
        write_sources(&self.pool, &name, files, message).map_err(Into::into)
    }

    fn merge_tables(
        &self,
        bot_id: uuid::Uuid,
        tables_bas: &str,
    ) -> Result<Vec<String>, botautotask::types::BoxError> {
        let name = bot_name_for_id(&self.pool, bot_id)?;
        merge_tables(&self.pool, &name, tables_bas).map_err(Into::into)
    }

    fn write_bot_config(
        &self,
        bot_id: uuid::Uuid,
        files: &[(String, String)],
        message: &str,
    ) -> Result<Vec<String>, botautotask::types::BoxError> {
        let name = bot_name_for_id(&self.pool, bot_id)?;
        super::super::bot_config::write_config_files(&self.pool, &name, files, message).map_err(Into::into)
    }

    fn read_source(
        &self,
        bot_id: uuid::Uuid,
        name: &str,
    ) -> Result<Option<String>, botautotask::types::BoxError> {
        let bot_name = bot_name_for_id(&self.pool, bot_id)?;
        read_source(&self.pool, &bot_name, name).map_err(Into::into)
    }

    fn list_sources(
        &self,
        bot_id: uuid::Uuid,
    ) -> Result<botautotask::types::SourceListing, botautotask::types::BoxError> {
        let bot_name = bot_name_for_id(&self.pool, bot_id)?;
        list_sources(&self.pool, &bot_name).map_err(Into::into)
    }
}
