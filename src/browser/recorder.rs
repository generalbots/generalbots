use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionType {
    Navigate,
    Click,
    Fill,
    Wait,
    Assert,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordedAction {
    pub timestamp: i64,
    pub action_type: ActionType,
    pub selector: Option<String>,
    pub value: Option<String>,
}

pub struct ActionRecorder {
    pub actions: Vec<RecordedAction>,
    pub is_recording: bool,
}

impl Default for ActionRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionRecorder {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            is_recording: false,
        }
    }

    pub fn start(&mut self) {
        self.is_recording = true;
    }

    pub fn stop(&mut self) -> Vec<RecordedAction> {
        self.is_recording = false;
        // drain returns an iterator, we can just return a clone or transfer ownership
        std::mem::take(&mut self.actions)
    }

    pub fn export_test_script(&self, recorded_actions: &[RecordedAction]) -> String {
        let mut script = String::from("import { test, expect } from '@playwright/test';\n\n");
        script.push_str("test('Recorded test', async ({ page }) => {\n");

        for action in recorded_actions {
            match action.action_type {
                ActionType::Navigate => {
                    if let Some(url) = &action.value {
                        script.push_str(&format!("  await page.goto('{}');\n", url));
                    }
                }
                ActionType::Click => {
                    if let Some(sel) = &action.selector {
                        script.push_str(&format!("  await page.click('{}');\n", sel));
                    }
                }
                ActionType::Fill => {
                    if let (Some(sel), Some(val)) = (&action.selector, &action.value) {
                        script.push_str(&format!("  await page.fill('{}', '{}');\n", sel, val));
                    }
                }
                _ => {}
            }
        }

        script.push_str("});\n");
        script
    }
}
