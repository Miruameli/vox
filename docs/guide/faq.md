# FAQ

### How is Vox different from the TypeScript Pi?

Vox is a native Rust reimplementation: one binary, in-process search and
editing, no Node runtime. The architecture follows Pi's separation of
providers, agent runtime, and tools; the feature set targets IDE-grade
breadth in a terminal. See [ADR-0001](../adr/foundation/0001-rust-native-reimplementation.md).

### Which providers work today?

Any OpenAI-compatible chat completions endpoint: OpenAI, OpenRouter, local
servers (Ollama, llama.cpp, vLLM, LM Studio), and most gateways. Anthropic
and Google adapters are planned ([ADR-0004](../adr/foundation/0004-provider-abstraction.md)).

### Where are API keys stored?

Nowhere in the repo. Keys are read from the environment variable named by
`general.api_key_env` (default `OPENAI_API_KEY`) at run time. Configuration
files containing credential-shaped values are rejected at load time.

### Why is my `vox.toml` rejected?

Unknown keys fail validation with the list of valid keys in the log. Run
`vox config` to print the resolved settings, and check the schema in
`crates/core/vox-config/src/schema.rs`.

### Why can I not use an `http://` base URL?

Plain HTTP is allowed only on loopback hosts so credentials never leave the
machine unencrypted. Use `https://` otherwise.

### Why did CI fail on a file I wrote?

The structure gate limits files to 150 lines of code and folders to 5 direct
files and 5 subfolders. Split the file or record an ADR justifying an
exception. See [ADR-0008](../adr/operations/0008-quality-gates.md).

### Why must every PR reference an issue?

Traceability: every change maps to a decision or a defect someone can read.
See [development-workflow.md](../governance/development-workflow.md).
