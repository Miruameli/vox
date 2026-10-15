# Glossary

Terms used across Vox documentation and code.

| Term | Meaning |
| --- | --- |
| Agent | The loop that drives a model: send context, receive tool calls, execute tools, repeat. |
| ADR | Architecture decision record; documents a significant decision with alternatives and consequences. |
| ACP | Agent Client Protocol; a JSON-RPC protocol editors use to drive an agent. |
| Hashline edit | An edit format where target lines are addressed by content hash; stale anchors are rejected before apply. |
| Layer | One of the architectural layers: domain, application, shared, config, infrastructure, presentation, interfaces. |
| Model provider | An adapter speaking one model API protocol (for example OpenAI-compatible chat completions). |
| Print mode | The `-p` entry point: one prompt, streamed answer, exit. |
| Role | Model routing intent (default, fast, deep, plan, review) mapped to a provider and model in configuration. |
| SSE | Server-Sent Events; the streaming transport used by chat completion APIs. |
| Structure gate | The CI check enforcing the file and folder limits of this repository. |
| Subagent | A spawned worker session with its own tool surface and typed result contract. |
| Trusted extension | A code-executing extension loaded only by explicit path, never ambient. |
