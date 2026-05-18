# cc-statusline

Fast, configurable statusline for [Claude Code](https://claude.com/claude-code), written in Rust.

## Highlights

- Single static binary (~700 KB stripped)
- Cold start under 3 ms with the default config, under 5 ms with all modules enabled
- TOML configuration with a starship-style format string
- 12 modules out of the box (Claude Code + git)
- Truecolor with automatic fallback (24-bit → 256 → 16) and `NO_COLOR` support
- Catches panics — the statusline will never visibly crash Claude Code

## Install

### One-liner

```sh
curl -fsSL https://raw.githubusercontent.com/mediavee/cc-statusline/main/install.sh | sh
```

This installs into `~/.local/bin` by default. Override with `INSTALL_DIR=...`.

### From source

```sh
cargo install --git https://github.com/mediavee/cc-statusline
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
| `rate_limits` | stdin | on |
| `output_style` | stdin | on |
| `version` | stdin | off |
| `cache_hit` | transcript | off |
| `transcript_stats` | transcript | off |
| `tool_usage` | transcript | off |

Run `cc-statusline modules` to list, or `cc-statusline modules <name>` for the variables a module exposes.

## Configuration

Config file is resolved in this order:

1. `$CC_STATUSLINE_CONFIG` (env)
2. `$XDG_CONFIG_HOME/cc-statusline/config.toml`
3. `~/.claude/cc-statusline.toml`

Example:

```toml
format = "$workspace[ $git_branch][ $git_status][ $model][ $context][ $cost]"

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
