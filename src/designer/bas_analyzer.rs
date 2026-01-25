use std::path::Path;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum BasFileType {
    Tool,
    Workflow,
    Regular,
}

pub struct BasFileAnalyzer;

impl BasFileAnalyzer {
    pub fn analyze_file(file_path: &Path) -> Result<BasFileType, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        Self::analyze_content(&content)
    }
    
    pub fn analyze_content(content: &str) -> Result<BasFileType, Box<dyn std::error::Error>> {
        let content_upper = content.to_uppercase();
        
        // Check for workflow keywords
        if content_upper.contains("ORCHESTRATE WORKFLOW") ||
           content_upper.contains("ON EVENT") ||
           content_upper.contains("PUBLISH EVENT") ||
           content_upper.contains("BOT SHARE MEMORY") ||
           content_upper.contains("WAIT FOR EVENT") {
            return Ok(BasFileType::Workflow);
        }
        
        // Check for tool patterns
        if Self::is_tool_pattern(&content_upper) {
            return Ok(BasFileType::Tool);
        }
        
        Ok(BasFileType::Regular)
    }
    
    fn is_tool_pattern(content: &str) -> bool {
        // Tool patterns: simple input/output, no complex orchestration
        let has_simple_structure = content.contains("WHEN") && 
                                 content.contains("DO") &&
                                 !content.contains("ORCHESTRATE");
        
        let has_tool_keywords = content.contains("USE TOOL") ||
                              content.contains("CALL TOOL") ||
                              content.contains("GET") ||
                              content.contains("SET");
        
        // Tools typically have fewer than 50 lines and simple logic
        let line_count = content.lines().count();
        let is_simple = line_count < 50;
        
        has_simple_structure && has_tool_keywords && is_simple
    }
    
    pub fn get_workflow_metadata(content: &str) -> WorkflowMetadata {
        let mut metadata = WorkflowMetadata::default();
        
        // Extract workflow name
        if let Some(start) = content.find("ORCHESTRATE WORKFLOW") {
            if let Some(name_start) = content[start..].find('"') {
                if let Some(name_end) = content[start + name_start + 1..].find('"') {
                    let name = &content[start + name_start + 1..start + name_start + 1 + name_end];
                    metadata.name = name.to_string();
                }
            }
        }
        
        // Count steps
        metadata.step_count = content.matches("STEP").count();
        
        // Count bots used
        metadata.bot_count = content.matches("BOT \"").count();
        
        // Check for human approval
        metadata.has_human_approval = content.contains("HUMAN APPROVAL");
        
        // Check for parallel processing
        metadata.has_parallel = content.contains("PARALLEL");
        
        metadata
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorkflowMetadata {
    pub name: String,
    pub step_count: usize,
    pub bot_count: usize,
    pub has_human_approval: bool,
    pub has_parallel: bool,
}
