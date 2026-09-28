# zeldoc

Command-line client for [Zeldoc.ai](https://zeldoc.ai), the EU-sovereign LLM API.

It is small on purpose. It exposes the parts of Zeldoc.ai that are useful from a shell,
and coding agents such as Claude Code, Codex, OpenCode or Pi can use it with no plugin:
any agent that can run a command can list models or search the web through Zeldoc.ai.

## Install

**macOS and Linux:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/martinmose/zeldoc-cli/releases/latest/download/zeldoc-installer.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/martinmose/zeldoc-cli/releases/latest/download/zeldoc-installer.ps1 | iex"
```

The installers download the prebuilt binary for your platform from the
[latest release](https://github.com/martinmose/zeldoc-cli/releases/latest), check its
checksum and put it in `~/.local/bin` (`%USERPROFILE%\.local\bin` on Windows), or in
`$XDG_BIN_HOME` when that is set, adding the directory to your `PATH` if needed. Open a
new terminal afterwards. No Rust toolchain is needed.

Releases have binaries for macOS (Apple silicon and Intel), Linux (x86_64 and ARM64) and
Windows (x86_64); you can also download an archive from the release page yourself. To
build from source instead: `cargo install --path .`

## Authenticate

Save an API key once. It is checked against Zeldoc.ai before it is stored:

```bash
zeldoc auth login
```

The key is read from a hidden prompt, or from standard input when input is piped
(`zeldoc auth login < key.txt`). Don't have a key yet? See
[Generate an API key](https://docs.zeldoc.ai/connect-opencode#generate-an-api-key).

`ZELDOC_API_KEY`, when set, takes precedence over the saved key, so existing setups keep
working without a login.

| Command | Does |
|---|---|
| `zeldoc auth login` | Checks and saves a key |
| `zeldoc auth status` | Shows which key is in use (masked) and whether it works; exits 1 if not |
| `zeldoc auth token` | Prints the key, for tools that read `ZELDOC_API_KEY` |
| `zeldoc auth logout` | Deletes the saved key |

### Where the key is stored

In `credentials.json` in the platform config directory:

| Platform | Path |
|---|---|
| Linux | `~/.config/zeldoc/credentials.json` |
| macOS | `~/Library/Application Support/zeldoc/credentials.json` |
| Windows | `%APPDATA%\zeldoc\credentials.json` |

`ZELDOC_CONFIG_DIR` overrides the directory. On Unix the file is created with mode `0600`
in a `0700` directory, so only your user can read it. It is not encrypted: like
`~/.codex/auth.json` or `~/.aws/credentials`, anything running as your user, including an
AI agent with shell access, can read it. The same is true of `ZELDOC_API_KEY` in your
environment.

### Using the saved key in other tools

`zeldoc auth token` prints the key the CLI uses: `ZELDOC_API_KEY` if it is set, otherwise
the key saved by `zeldoc auth login`. It does not create a new key and makes no network
call, so it is cheap to run every time a shell starts.

A program cannot set environment variables in the shell that started it, so
`zeldoc auth login` cannot set `ZELDOC_API_KEY` for you. Set it from `zeldoc auth token`
in your shell profile instead, so every new shell has it. To set it in the current shell
only, run the same line there.

**Linux and macOS (bash, zsh).** Add this line to `~/.bashrc` or `~/.zshrc` (zsh is the
default shell on macOS):

```bash
export ZELDOC_API_KEY="$(zeldoc auth token)"
```

**fish.** Add this line to `~/.config/fish/config.fish`:

```fish
set -gx ZELDOC_API_KEY (zeldoc auth token)
```

**Windows (PowerShell).** Add the line to your PowerShell profile, creating the profile
first if you don't have one:

```powershell
if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE -Force | Out-Null }
Add-Content -Path $PROFILE -Value '$env:ZELDOC_API_KEY = zeldoc auth token'
```

If new PowerShell windows then report that running scripts is disabled, the execution
policy is blocking your profile. `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`
allows local scripts such as the profile to run.

**Programs started from the Start menu or the Dock** don't read shell profiles. On Windows,
set a user environment variable instead:

```powershell
[Environment]::SetEnvironmentVariable("ZELDOC_API_KEY", (zeldoc auth token), "User")
```

This stores a copy of the key in your Windows user environment. Run it again after you
change key, then restart the programs that use it.

Setting the variable from `zeldoc auth token` keeps the key out of your profile file,
which often ends up in a dotfiles repository. The key is still in the environment of every
program started from that shell, as with a hand-written `export`.

After `zeldoc auth login` with a new key, open a new shell: shells that are already open
keep the old value, and `ZELDOC_API_KEY` takes precedence over the saved key.

## List models

```bash
zeldoc models
zeldoc models --mode chat
zeldoc models --json
```

Lists the models your key can use from Zeldoc.ai's model catalog
(`GET /v1/zeldoc/models`): kind, context window, output limit, your organization's price
per 1 million tokens, and capabilities. `--json` adds cache prices and the accepted
`reasoning_effort` values. `--mode` filters on the kind: `chat`, `responses`,
`embedding`, `image_generation`, `audio_transcription` or `realtime`.

The gateway's own `GET /v1/models` only lists names; it has no limits or prices for
aliases such as `zdev`, which is why the CLI uses the catalog.

## Search the web

```bash
zeldoc search podman rootless ports below 1024
zeldoc search --engines github,stackoverflow -n 5 axum middleware
zeldoc search --json --time-range week rust 2024 edition
```

Calls Zeldoc.ai's search endpoint (`POST /v1/search/zeldoc-search`). General searches use
Staan, an EU-based provider; `--categories it` and `--engines` can reach services outside
the EU. See [Web search](https://docs.zeldoc.ai/web-search) for what leaves your machine,
and keep personal or confidential information out of queries.

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| `ZELDOC_API_KEY` | | API key; overrides the saved key |
| `ZELDOC_CONFIG_DIR` | platform config directory + `/zeldoc` | Where the saved key lives |

## License

[MIT](LICENSE)
