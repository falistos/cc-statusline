# cc-statusline

Fast, configurable statusline for [Claude Code](https://claude.com/claude-code), written in Rust.

## Highlights

- Single static binary (~950 KB stripped)
- ~2 ms per render with the default config, cached git status included
- TOML configuration with a starship-style format string; a newline in the
  format string starts a second statusline row
- 14 modules out of the box (Claude Code + git)
- Usage windows survive the payload going quiet, and expose the model-scoped
  weekly window Claude Code never sends
- Truecolor with automatic fallback (24-bit → 256 → 16) and `NO_COLOR` support
- Catches panics — the statusline will never visibly crash Claude Code

## Install

### One-liner

```sh
curl -fsSL https://raw.githubusercontent.com/falistos/cc-statusline/main/install.sh | sh
```

This installs into `~/.local/bin` by default. Override with `INSTALL_DIR=...`.

### From source

```sh
cargo install --git https://github.com/falistos/cc-statusline
```

### Setup

```sh
cc-statusline init
```

Writes a starter config and patches `~/.claude/settings.json` to point at the binary. Existing settings are backed up to `settings.json.bak`.

## Modules

| Module | Source | Default |
|---|---|---|
| `workspace` | stdin | on |
| `git_branch` | raw `.git/HEAD` | on |
| `git_status` | shell + 5s cache | on |
| `model` | stdin | on |
| `context` | stdin / transcript / auto | on |
| `cost` | stdin | on |
| `rate_limits` | stdin + store + `~/.claude.json` | on |
| `prompt_cache` | stdin (needs Claude Code ≥ 2.1.251) | on |
| `session` | stdin | on |
| `output_style` | stdin | on |
| `version` | stdin | off |
| `cache_hit` | transcript | off |
| `transcript_stats` | transcript | off |
| `tool_usage` | transcript | off |

Run `cc-statusline modules` to list, or `cc-statusline modules <name>` for the variables a module exposes.

## Usage windows

`rate_limits` merges three sources, freshest first, because Claude Code only
fills `rate_limits` in the payload right after an API response — most renders
receive nothing:

1. the payload itself
2. `~/.claude/cc-statusline/cache/windows.json`, the last payload values seen,
   written back by the module
3. `cachedUsageUtilization` in `~/.claude.json`, the only source for the
   model-scoped weekly window (`$scoped`, e.g. Fable) and the extra-credit
   balance (`$credits`)

Values that did not come from the current payload and are older than
`stale_after_seconds` are prefixed with `stale_symbol` (`~` by default), and a
window whose reset time has passed is dropped rather than shown stale. Reset
countdowns only appear once a window is at or above `reset_above_percent`, or
rolls over within `reset_within_seconds`.

## Prompt cache

`prompt_cache` reports the health of the conversation prefix: hit ratio, and
how long before the cached prefix goes cold — after which the next request
pays full price. The icon and the countdown escalate together (`warm_symbol` →
`alert_symbol` under `alert_seconds`, `cold_symbol` once cold). The module is
silent when the payload has no `prompt_cache` object, which is the case before
Claude Code 2.1.251 and behind gateways that strip cache token counts.

## Configuration

Config file is resolved in this order:

1. `$CC_STATUSLINE_CONFIG` (env)
2. `$XDG_CONFIG_HOME/cc-statusline/config.toml`
3. `~/.claude/cc-statusline.toml`

Example:

```toml
# A newline starts a second row: identity on top, spend below.
format = "$workspace[  $git_branch][ $git_status][  $model][  $session]\n$context[  $prompt_cache][  $rate_limits][  $cost]"

[model]
style = "cyan"
[model.aliases]
"claude-opus-4-7" = "opus-4.7"

[context]
source = "auto"  # stdin | transcript | auto
[[context.thresholds]]
max = 50
style = "green"
[[context.thresholds]]
max = 80
style = "yellow"
[[context.thresholds]]
max = 100
style = "red bold"

[git_status]
cache_ttl_seconds = 5
show_counts = true
```

### Format grammar

- `$name` — variable / module reference
- `[ ... ]` — group (collapses entirely if no variable inside renders)
- `[ ... ]($style)` — styled group
- `\$ \[ \] \\` — escapes

### Styles

`"bold red"`, `"red on blue"`, `"#ff8800"`, `"bg:#222 fg:white italic"`, `"244"` (ANSI 256), `"dim"`, `"underline"`, `"none"`.

## CLI

```
cc-statusline                 read stdin JSON, write statusline
cc-statusline init [--force]  create config + patch settings.json
cc-statusline validate        parse and lint the current config
cc-statusline modules [name]  list modules or describe one
cc-statusline preview         render with a mock payload
```

## License

Proprietary — Mediavee.
