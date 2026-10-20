# Controlling AoE over MCP

The daemon can serve a [Model Context Protocol](https://modelcontextprotocol.io) server, so an agent can work with AoE Sessions through typed tools instead of scraping `aoe` CLI output. This is separate from [MCP Servers](mcp-servers.md), which forwards your own servers to the agents AoE launches.

## Turn it on

The server is off by default. Turn on the **MCP server** toggle in Settings, or add this to `config.toml`:

```toml
[web]
mcp_enabled = true
```

The daemon reads the setting at startup, so restart it afterwards with `aoe serve --restart`, or start one with `aoe serve --daemon`. It then serves MCP over Streamable HTTP at `/mcp` on the same port as the dashboard and API.

## Connect an agent

On the daemon's host, run:

```bash
aoe mcp config
```

It prints an `mcpServers` entry for the running daemon: the loopback `/mcp` URL and, when the daemon uses token authentication, an `Authorization: Bearer` header carrying its current token.

```json
{
  "mcpServers": {
    "aoe": {
      "type": "http",
      "url": "http://127.0.0.1:8080/mcp",
      "headers": {
        "Authorization": "Bearer <token>"
      }
    }
  }
}
```

Paste it into the agent's MCP configuration, for example a Claude Code `.mcp.json` in your project. The output contains the daemon token, so keep it out of version control.

Before printing, the command asks the running daemon whether it serves `/mcp`, because the daemon applies `mcp_enabled` only at startup. If you turned the setting on after the daemon started, it warns that `/mcp` is not served yet and that `aoe serve --restart` applies it; if you turned it off, it notes that the route goes away at the next restart. Without a running daemon it exits non-zero and tells you how to start one.

### Authentication modes

| Daemon | Entry |
| --- | --- |
| Token authentication (the default, including `--remote`) | URL plus the `Authorization: Bearer` header |
| `--auth passphrase` without `--behind-proxy` | URL only; loopback callers bypass the login wall |
| `--auth passphrase --behind-proxy` | None: the login wall covers loopback, and `/mcp` accepts only a browser login session. The command exits non-zero; restart the daemon with `--auth token`, or without `--behind-proxy` if no proxy fronts it |
| `--auth none` | URL only |

If the daemon refuses the printed credential, for example because the token rotated at that moment, the command exits non-zero instead of printing an entry that cannot connect.

## Tools

The tools and their arguments are listed in the [HTTP API reference](../api.md#mcp-server). Every tool returns structured content with an output schema.

## Known limits

- The daemon must already be running. Connecting an MCP client does not start it.
- `aoe mcp config` targets the daemon on this machine through its loopback address. It does not read `--daemon-url` or `AOE_DAEMON_URL`.
- Under `aoe serve --remote` the token rotates every 4 hours, and a pasted entry stops working after the rotation grace period. Run `aoe mcp config` again to get the current token.
- An MCP client has the same access as you at the keyboard. AoE adds no per-tool permissions; the agent's own tool rules decide what it may call.
