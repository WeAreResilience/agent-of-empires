# The Session Board is a plugin; core gains sandboxed plugin pages

Status: proposed. Decides #4433. Terms follow `CONTEXT.md`: **Board**, **Column**, **Card**, **Stage**, **Signal**, **Rule**; a **Plugin** contributes **UI slots**.

Issue #3446 settled on a plugin-first Board, and the read-only `kanban-lite` prototype (#3456) stalled because plugins had no page surface, no way to write per-session state from the browser, and no access to other plugins' data. We considered a native core Board and rejected it: the Board serves a subset of users and would add product surface to core that maintainers do not want to own. Instead core grows one general capability, a `page` UI slot: a plugin ships a single self-contained HTML file, the Web dashboard renders it in a sandboxed iframe (no `allow-same-origin`, loaded through `srcdoc` from an authenticated fetch), and a versioned `postMessage` bridge exposes a capability-gated API (session list, plugin UI state, the plugin's own actions, open-session navigation, archive and trash under `session.write`, theme variables). The Board (`agent-of-empires.kanban`) is the first consumer.

## Consequences

- Plugin JavaScript never runs on the dashboard origin and never sees the access token; everything it can do is enumerated by the bridge and gated by manifest capabilities.
- Writes from a page are fire-and-forget through the existing plugin `action` endpoint; results surface through the UI-state poll. A request/response channel is deferred until a plugin needs it.
- Per-session plugin state lives in `session.meta`; plugin-wide state in `plugin.storage`. Core gains no Board-specific schema or migration.
- Confirms direction A of #3446 with the extension points it anticipated.
