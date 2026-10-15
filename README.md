<p align="center">
  <img alt="Vox logo" src="assets/logo.svg" width="480">
</p>

<p align="center">
  <strong>A complete terminal coding agent, native Rust.</strong>
</p>

<p align="center">
  <a href="https://github.com/Miruameli/vox"><img alt="GitHub Miruameli" src="assets/badge.svg" height="28"></a>
  <a href="https://github.com/Miruameli/vox/blob/main/LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-C9A227?style=flat-square&labelColor=4A1525"></a>
  <a href="https://github.com/Miruameli/vox/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/Miruameli/vox/ci.yml?style=flat-square&label=ci&labelColor=4A1525&color=C9A227"></a>
</p>

> Status: early development. The Rust-native reimplementation has started
> from a blueprint. See [docs/index.md](docs/index.md) for the plan and
> roadmap.
>
> Status: pengembangan awal. Reimplementasi Rust-native dimulai dari
> blueprint. Lihat [docs/index.md](docs/index.md) untuk rencana dan roadmap.

**Vox** mengimplementasikan ulang arsitektur coding agent Pi sebagai produk
Rust native: satu binary, hot path in-proses, tanpa runtime Node. Target
fiturnya setara IDE modern — pencarian dan penyuntingan in-proses, intelijensi
language server, subagent, sesi durable — tetapi tetap coding terminal saja.

Vox reimplements the Pi coding agent architecture as a native Rust product:
one binary, in-process hot paths, no Node runtime. The feature set targets
IDE-grade breadth — in-process search and editing, language-server
intelligence, subagents, durable sessions — but stays a coding terminal.

## Fitur yang direncanakan / planned features

- Print mode streaming (`vox -p`) terhadap provider OpenAI-compatible — sudah jalan
- Edit engine ber-anchor content hash (hashline), apply atomik — sudah jalan
- Tool harness: read/write/edit/grep/glob/bash + LSP + DAP
- Subagent, sesi durable, konfigurasi berlapis dengan model kepercayaan eksplisit
- TUI interaktif, ACP/RPC untuk editor

## Build / Menjalankan

```sh
git clone https://github.com/Miruameli/vox.git
cd vox
cargo build --workspace
cargo test --workspace
cargo run -p vox-interfaces-cli -- --help
```

Contoh print mode (butuh API key di environment):

```sh
OPENAI_API_KEY=... cargo run -p vox-interfaces-cli -- \
  -p "jelaskan arsitektur repo ini"
```

Dengan server lokal (Ollama, llama.cpp, vLLM):

```sh
cargo run -p vox-interfaces-cli -- \
  --base-url http://127.0.0.1:11434/v1 --api-key-env OLLAMA_API_KEY \
  -p "ringkas file README ini"
```

## Referensi / References

- Architecture reference: `earendil-works/pi` (TypeScript original)
- Feature reference: `can1357/oh-my-pi` (IDE-wired agent, Rust native crates)

Clean-room reimplementation; no code is copied from either reference. Design
decisions are documented in [docs/adr/](docs/adr/).

## Dokumentasi / Documentation

- [docs/index.md](docs/index.md) — peta dokumentasi
- [docs/guide/onboarding.md](docs/guide/onboarding.md) — mulai di sini
- [docs/guide/environment.md](docs/guide/environment.md) — variabel environment
- [docs/governance/](docs/governance/) — contributing, workflow, security
- [docs/runbook/](docs/runbook/) — development dan release

## Lisensi / License

MIT. Lihat [LICENSE](LICENSE).
