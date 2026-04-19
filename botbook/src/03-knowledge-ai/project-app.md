# Project App User Guide

The Project app is a comprehensive project management tool built into General Bots, providing Gantt charts, task dependencies, resource management, and critical path analysis.

## Overview

The Project app enables you to:

- Create and manage projects with tasks and milestones
- Visualize timelines with interactive Gantt charts
- Define task dependencies and relationships
- Allocate and track resource assignments
- Calculate critical paths for schedule optimization
- Track progress and completion status

## Getting Started

### Creating a Project

1. Navigate to **Apps** → **Project**
2. Click **New Project**
3. Enter project details:
   - **Name**: Project title
   - **Start Date**: When work begins
   - **End Date**: Target completion (optional)
   - **Description**: Project overview

### Project Structure

Projects contain:

```
Project
├── Summary Tasks (Phases)
│   ├── Tasks
│   │   ├── Subtasks
│   │   └── Milestones
│   └── More Tasks
└── Milestones
```

## Working with Tasks

### Creating Tasks

Click **Add Task** or press `Enter` on the last task row:

| Field | Description |
|-------|-------------|
| Name | Task title |
| Duration | Time to complete (e.g., "5 days") |
| Start | Task start date |
| Finish | Task end date (calculated or manual) |
| Predecessors | Tasks that must complete first |
| Resources | Assigned team members |
| Progress | Completion percentage (0-100%) |

### Task Types

| Type | Icon | Description |
|------|------|-------------|
| Task | ▬ | Standard work item |
| Milestone | ◆ | Zero-duration checkpoint |
| Summary | ▬▬ | Container for subtasks |

### Duration Formats

| Format | Example | Result |
|--------|---------|--------|
| Days | `5d` or `5 days` | 5 working days |
| Weeks | `2w` or `2 weeks` | 10 working days |
| Hours | `16h` or `16 hours` | 2 working days |
| Minutes | `240m` | 4 hours |

## Gantt Chart

### Navigation

- **Scroll**: Mouse wheel or drag timeline
- **Zoom**: Ctrl + scroll or zoom buttons
- **Today**: Click "Today" to center on current date

### Time Scales

| View | Shows | Best For |
|------|-------|----------|
| Day | Hours | Short tasks |
| Week | Days | Sprint planning |
| Month | Weeks | Project overview |
| Quarter | Months | Portfolio view |
| Year | Quarters | Long-term planning |

### Bar Colors

| Color | Meaning |
|-------|---------|
| Blue | Normal task |
| Green | Completed task |
| Red | Critical path task |
| Orange | Behind schedule |
| Purple | Milestone |
| Gray | Summary task |

## Task Dependencies

### Dependency Types

| Type | Code | Description |
|------|------|-------------|
| Finish-to-Start | FS | B starts when A finishes |
| Start-to-Start | SS | B starts when A starts |
| Finish-to-Finish | FF | B finishes when A finishes |
| Start-to-Finish | SF | B finishes when A starts |

### Creating Dependencies

**Method 1: Predecessors Column**

Enter task numbers in the Predecessors column:

```
2          → Task 2 must finish first (FS)
2FS        → Same as above (explicit)
2SS        → Start when task 2 starts
2FF+2d     → Finish when task 2 finishes, plus 2 days
3,5        → Both tasks 3 and 5 must finish
```

**Method 2: Drag in Gantt**

1. Hover over a task bar
2. Drag from the end connector
3. Drop on another task

### Lag and Lead Time

| Syntax | Meaning |
|--------|---------|
| `2FS+3d` | Start 3 days after task 2 finishes |
| `2FS-2d` | Start 2 days before task 2 finishes |
| `2SS+1w` | Start 1 week after task 2 starts |

## Resource Management

### Adding Resources

1. Click **Resources** tab
2. Click **Add Resource**
3. Enter resource details:
   - Name
   - Type (Work, Material, Cost)
   - Rate (hourly/daily cost)
   - Availability (percentage)

### Resource Types

| Type | Use Case | Unit |
|------|----------|------|
| Work | People | Hours |
| Material | Consumables | Quantity |
| Cost | Fixed costs | Currency |

### Assigning Resources

In the Resources column, enter:

```
John              → 100% allocation
John[50%]         → 50% allocation
John,Sarah        → Two resources
John[50%],Sarah   → Mixed allocations
```

### Resource Views

- **Resource Sheet**: List all resources with details
- **Resource Usage**: Time-phased work assignments
- **Resource Graph**: Visual workload chart

## Critical Path

### Understanding Critical Path

The critical path is the longest sequence of dependent tasks that determines the minimum project duration. Tasks on the critical path have zero float—any delay extends the project.

### Viewing Critical Path

1. Click **Format** → **Critical Path**
2. Critical tasks appear in red
3. Hover for float information

### Critical Path Analysis

| Metric | Description |
|--------|-------------|
| Total Float | Time a task can slip without delaying project |
| Free Float | Time a task can slip without delaying successors |
| Critical | Float = 0, on critical path |

## Timeline View

The Timeline view provides a high-level summary:

1. Click **View** → **Timeline**
2. Drag tasks to add to timeline
3. Resize to set date range

### Timeline Features

- Executive summary view
- Milestone emphasis
- Export to PowerPoint
- Shareable as image

## Progress Tracking

### Updating Progress

**Method 1: Percentage**

Enter completion percentage (0-100%) in Progress column.

**Method 2: Actual Dates**

- Actual Start: When work actually began
- Actual Finish: When work actually completed

**Method 3: Remaining Work**

Enter remaining hours/days to completion.

### Status Indicators

| Indicator | Meaning |
|-----------|---------|
| ✓ | Complete (100%) |
| ● | In Progress |
| ○ | Not Started |
| ⚠ | Behind Schedule |
| 🔴 | Critical & Late |

### Baseline Comparison

1. Set baseline: **Project** → **Set Baseline**
2. View variance: **View** → **Tracking Gantt**
3. Gray bars show original plan

## Reports

### Built-in Reports

| Report | Shows |
|--------|-------|
| Project Summary | Overall status, milestones |
| Task Status | All tasks with progress |
| Resource Overview | Assignments and workload |
| Cost Report | Budget vs actual costs |
| Critical Tasks | Critical path analysis |

### Generating Reports

1. Click **Reports**
2. Select report type
3. Choose date range
4. Click **Generate**

### Export Options

- PDF
- Excel
- CSV
- Image (PNG)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Add new task |
| `Tab` | Indent task (make subtask) |
| `Shift+Tab` | Outdent task |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Delete` | Delete selected |
| `Ctrl+C` | Copy task |
| `Ctrl+V` | Paste task |
| `F5` | Go to date |
| `Ctrl+G` | Go to task |

## BASIC Integration

### Creating Projects from BASIC

```bas
project = CREATE PROJECT "Website Redesign"
project.start_date = "2025-02-01"
project.description = "Complete website overhaul"

task1 = CREATE TASK project, "Design Phase"
task1.duration = "10 days"

task2 = CREATE TASK project, "Development"
task2.duration = "20 days"
task2.predecessors = task1.id

milestone = CREATE MILESTONE project, "Launch"
milestone.predecessors = task2.id
```

### Querying Projects

```bas
projects = GET FROM projects WHERE status = "active"
FOR EACH project IN projects
    tasks = GET FROM tasks WHERE project_id = project.id
    TALK "Project: " + project.name + " has " + LEN(tasks) + " tasks"
NEXT
```

## Tips and Best Practices

### Planning

1. **Start with milestones** - Define key deliverables first
2. **Work backwards** - From deadline to start
3. **Break down tasks** - No task longer than 2 weeks
4. **Identify dependencies** - What blocks what?
5. **Add buffer** - Include contingency time

### Execution

1. **Update regularly** - Daily or weekly progress
2. **Monitor critical path** - Watch for delays
3. **Rebalance resources** - Address overallocation
4. **Communicate changes** - Keep stakeholders informed

### Common Mistakes

| Mistake | Solution |
|---------|----------|
| Missing dependencies | Review task relationships |
| Over-allocated resources | Level workload |
| No milestones | Add checkpoints |
| Too much detail | Summarize minor tasks |
| No baseline | Set before execution |

## Related Topics

- [Task Management](./tasks.md) - Task app integration
- [Calendar Integration](./calendar.md) - Scheduling
- [Forms Integration](./forms.md) - Task intake forms
- [BASIC Reference](../06-gbdialog/keywords.md) - Automation keywords