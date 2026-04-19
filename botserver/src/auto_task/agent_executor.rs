use crate::auto_task::container_session::{ContainerSession, TerminalOutput};
use crate::core::shared::state::{AppState, TaskProgressEvent};
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AgentExecutor {
    pub state: Arc<AppState>,
    pub session_id: String,
    pub task_id: String,
    container: Option<ContainerSession>,
}

impl AgentExecutor {
    pub fn new(state: Arc<AppState>, session_id: &str, task_id: &str) -> Self {
        Self {
            state,
            session_id: session_id.to_string(),
            task_id: task_id.to_string(),
            container: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), String> {
        self.broadcast_step("Initializing Agent Environment", 1, 10);
        
        let mut session = ContainerSession::new(&self.session_id).await?;
        let (tx, mut rx) = mpsc::channel(100);
        
        session.start_terminal(tx).await?;
        self.container = Some(session);

        // Spawn a task to listen to terminal output and broadcast it
        let state_clone = self.state.clone();
        let task_id_clone = self.task_id.clone();
        
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                let (line, stream) = match output {
                    TerminalOutput::Stdout(l) => (l, "stdout"),
                    TerminalOutput::Stderr(l) => (l, "stderr"),
                };
                
                let mut event = TaskProgressEvent::new(
                    &task_id_clone,
                    "terminal_output",
                    &line,
                )
                .with_event_type("terminal_output");
                // The JS on the frontend expects { type: "terminal_output", line: "...", stream: "..." }
                // So we hijack details for stream
                event.details = Some(stream.to_string());
                event.text = Some(line.clone());
                
                state_clone.broadcast_task_progress(event);
            }
        });

        self.broadcast_browser_ready("", 8000);
        self.broadcast_step("Agent Ready", 2, 10);

        Ok(())
    }

    pub async fn execute_shell_command(&mut self, cmd: &str) -> Result<(), String> {
        if let Some(container) = &mut self.container {
            container.send_command(cmd).await?;
            Ok(())
        } else {
            Err("Container not initialized".into())
        }
    }

    pub fn broadcast_thought(&self, thought: &str) {
        let mut event = TaskProgressEvent::new(
            &self.task_id,
            "thought_process",
            thought,
        )
        .with_event_type("thought_process");
        event.text = Some(thought.to_string());
        self.state.broadcast_task_progress(event);
    }

    pub fn broadcast_step(&self, label: &str, current: u8, total: u8) {
        let event = TaskProgressEvent::new(
            &self.task_id,
            "step_progress",
            label,
        )
        .with_event_type("step_progress")
        .with_progress(current, total);
        self.state.broadcast_task_progress(event);
    }

    pub fn broadcast_browser_ready(&self, url: &str, port: u16) {
        let mut event = TaskProgressEvent::new(
            &self.task_id,
            "browser_ready",
            url,
        )
        .with_event_type("browser_ready");
        event.details = Some(port.to_string());
        self.state.broadcast_task_progress(event);
    }

    pub async fn cleanup(&mut self) {
        if let Some(mut container) = self.container.take() {
            if let Err(e) = container.stop().await {
                error!("Error stopping container session: {}", e);
            }
        }
    }
}
