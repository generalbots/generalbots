# Workspace Tabs

With tabs enabled the chat surface gains a browser-style strip: conversations,
history items and apps live in renamable, persistent tabs.

- Open history in a new tab from the sidebar context menu.
- Each tab owns its own `session_id`; WebSocket frames are multiplexed with a
  `tabId` field and unread badges update per tab.
- State persists locally (`gb.tabs.v1`) and server-side via
  `GET|PUT /api/user/workspace/tabs`.

Tabs stay off by default: activating the picker, opening history into a tab,
or `?tabs=1` enables the shell. With a single default tab the payloads are
byte-identical to the classic interface.
