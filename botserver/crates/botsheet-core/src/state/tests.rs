use super::*;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct MemDrive(Mutex<HashMap<String, Vec<u8>>>);

#[async_trait]
impl DriveOps for MemDrive {
    async fn put_object(
        &self,
        _bucket: &str,
        key: &str,
        body: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), String> {
        self.0.lock().unwrap().insert(key.to_string(), body);
        Ok(())
    }

    async fn get_object(&self, _bucket: &str, key: &str) -> Result<Vec<u8>, String> {
        self.0
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| format!("no such object {key}"))
    }

    async fn list_objects(&self, _bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
        let map = self.0.lock().unwrap();
        let mut keys: Vec<String> = map
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        keys.sort();
        Ok(keys)
    }

    async fn delete_object(&self, _bucket: &str, key: &str) -> Result<(), String> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}

fn sample_sheet(owner: &str) -> Spreadsheet {
    let mut acl = HashMap::new();
    acl.insert("alice".to_string(), "edit".to_string());
    Spreadsheet {
        id: "s1".to_string(),
        name: "Report".to_string(),
        owner_id: owner.to_string(),
        worksheets: vec![crate::types::Worksheet::default()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl,
    }
}

#[tokio::test]
async fn save_writes_sidecar_and_list_uses_it() {
    let drive = Arc::new(MemDrive::default());
    let state = SheetState {
        drive: Some(drive.clone()),
        on_save: None,
        sessions: SessionStore::default(),
        metrics_collector: Arc::new(botcore::shared::analytics::MetricsCollector::new()),
    };
    let sheet = sample_sheet("alice");
    save_sheet_to_drive(&state, "alice", &sheet).await.unwrap();

    {
        let map = drive.0.lock().unwrap();
        let meta_key: Vec<&String> = map.keys().filter(|k| k.ends_with(".meta.json")).collect();
        assert_eq!(meta_key.len(), 1, "save writes exactly one sidecar");
        let meta: SpreadsheetMetadata =
            serde_json::from_slice(&map[meta_key[0]]).expect("sidecar parses");
        assert_eq!(meta.id, "s1");
        assert_eq!(meta.worksheet_count, 1);
        assert_eq!(meta.acl.get("alice").map(String::as_str), Some("edit"));
    }

    let listed = list_sheets_from_drive(&state, "alice").await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "Report");
}

#[tokio::test]
async fn list_loads_legacy_documents_without_sidecar() {
    let drive = Arc::new(MemDrive::default());
    let state = SheetState {
        drive: Some(drive.clone()),
        on_save: None,
        sessions: SessionStore::default(),
        metrics_collector: Arc::new(botcore::shared::analytics::MetricsCollector::new()),
    };
    let sheet = sample_sheet("alice");
    let path = format!("{}/{}.json", get_user_sheets_path("alice"), sheet.id);
    let content = serde_json::to_string(&sheet).unwrap();
    drive
        .put_object("gbo", &path, content.into_bytes(), "application/json")
        .await
        .unwrap();

    let listed = list_sheets_from_drive(&state, "alice").await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "Report");
}

#[tokio::test]
async fn delete_removes_sidecar() {
    let drive = Arc::new(MemDrive::default());
    let state = SheetState {
        drive: Some(drive.clone()),
        on_save: None,
        sessions: SessionStore::default(),
        metrics_collector: Arc::new(botcore::shared::analytics::MetricsCollector::new()),
    };
    let sheet = sample_sheet("alice");
    save_sheet_to_drive(&state, "alice", &sheet).await.unwrap();
    delete_sheet_from_drive(&state, "alice", "s1")
        .await
        .unwrap();
    let map = drive.0.lock().unwrap();
    assert!(map.is_empty(), "delete clears json, sidecar and xlsx");
}
