<p align="center"><strong>Anzoth CLI</strong> is a coding agent that runs locally on your computer.
<p align="center">
  <img src="https://github.com/lowenmk/anzoth-cli/blob/main/.github/codex-cli-splash.png" alt="Anzoth CLI splash" width="80%" />
</p>
</br>
If you want Anzoth in your code editor, install the matching IDE extension.
</br>If you want the desktop app experience, run <code>anzoth app</code> or visit the Anzoth App page.
</br>If you are looking for the cloud-based agent from OpenAI, go to chatgpt.com/codex.</p>

---

## Quickstart

### Installing and running Anzoth CLI

Run the following on Mac or Linux to install Anzoth CLI:

```shell
curl -fsSL https://chatgpt.com/codex/install.sh | sh
```

Run the following on Windows to install Anzoth CLI:

```shell
powershell -ExecutionPolicy ByPass -c "irm https://chatgpt.com/codex/install.ps1 | iex"
```

Codex CLI can also be installed via the following package managers:

```shell
# Install using npm
npm install -g @anzoth/cli
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `anzoth` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/lowenmk/anzoth-cli/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `anzoth-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `anzoth-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `anzoth-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `anzoth-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `anzoth-x86_64-unknown-linux-musl`), so you likely want to rename it to `anzoth` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `anzoth` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Anzoth as part of your Plus, Pro, Business, Edu, or Enterprise plan.

You can also use Anzoth with an API key, but this requires additional setup.

## Docs

- [**Anzoth Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
