# Agent Workspaces

General Bots supports an advanced "Agent Mode" UI where the interaction with an AI agent shifts from a simple chat interface to a fully realized multi-panel workspace. This design empowers users to monitor complex autonomous tasks in real-time, providing deep visibility into what the AI is doing.

## Architectural Overview

When an application logic branch triggers a computationally heavy or open-ended action (like `APP_CREATE` or deep research), the backend kicks off an **Agent Executor** process. This executor brings up a dedicated, highly isolated LXC container for that specific session.

The Agent Executor communicates back to the frontend in real-time. This provides a transparent view of:

1. **Thought Processes**: High-level textual reasoning of the agent.
2. **Terminal Output**: Verbatim standard output and standard error from the LXC container.
3. **Browser Output**: Visual previews of web applications being built or research being conducted, served via localhost proxies from within the container.

## The Agent UI

The main interface pivots from a standard 1-panel conversation to a complex multi-panel grid when "Agent Mode" is toggled from the Chat interface.

This layout includes:

- **Left Sidebar**: A collapsible "Agents & Workspaces" sidebar that summarizes the current state of active agents, their resource usage (quota), and provides drag-and-drop workspace organization.
- **Center Chat**: A persistent interactive chat with the specific agent, allowing for ongoing refinement of the task.
- **Right Hand Split Screens**: 
    - **Top Right**: An active Browser Window. The agent can stream HTML rendering updates or host internal applications (`localhost`) from its LXC sandbox, exposing them visually to the user.
    - **Bottom Right**: A live Terminal feed streaming `stdout` and `stderr` directly from the bash environment of the underlying LXC container.

## LXC Sandbox execution

To prevent dependency collisions, protect the host operating system, and offer clean slate environments for arbitrary execution, every agent session spins up a temporary **Ubuntu 22.04 LXC container**.

1. When the agent intent classifier matches a heavy task (e.g. `APP_CREATE`), the backend initiates the `ContainerSession` struct.
2. An `lxc launch` command instantiates a fast, lightweight container instance.
3. A bash shell is opened inside this container, and its I/O streams are piped back to the `TaskProgressEvent` broadcast channel using Tokio.
4. The user sees the bash output instantly in the bottom-right terminal panel.
5. On completion or failure, the container is forcibly stopped and deleted (`lxc delete --force`).

This isolated environment gives agents the absolute freedom to execute package installations (like `npm install`), launch development servers, and write arbitrary code, entirely segregated from the primary `BotServer`.
