# ADR-0004: Provider abstraction, OpenAI-compatible first

- Status: Accepted
- Date: 2026-10-15

## Context

Vox must talk to many model providers. The references support 60+ providers
including OpenAI, Anthropic, Google, OpenRouter, and OpenAI-compatible local
servers (Ollama, llama.cpp, vLLM, LM Studio). Provider protocols differ:
chat completions, responses, Anthropic messages, Google generative AI.

## Decision

Define a provider trait in `vox-domain` with three operations: list models,
start a completion (non-streaming), and stream a completion. The trait is
transport-agnostic and returns normalized `Turn`, `Message`, and `ToolCall`
domain types.

Implementations live in `platform/vox-provider/`:

1. `vox-provider-openai` first: covers OpenAI-compatible chat completions,
   which also serves OpenRouter, local servers, and most gateways. Streaming
   via SSE with incremental delta parsing.
2. `vox-provider-anthropic` next: Anthropic messages protocol.
3. `vox-provider-google` after: generative AI protocol.

Model catalog: a static bundled catalog file (provider, model id, context
window, capabilities) plus user-defined custom providers in configuration.
Roles (default, fast, deep, plan, review) map to provider/model selectors in
configuration, not in code.

## Alternatives

1. Ship all providers at once.
   Rejected: untestable breadth before the core loop exists; breadth without
   the agent loop is dead code.
2. Vendor a Rust SDK per provider.
   Rejected: dependency explosion, weak streaming control, slow updates.
3. One HTTP client with protocol adapters (chosen): one stack, three protocol
   adapters, per-provider tests against protocol fixtures.

## Consequences

- Adding a provider means adding an adapter crate plus fixtures, not touching
  the agent loop.
- Auth handling (API key, OAuth where required) sits in the provider crate,
  reading credentials from environment or the OS credential store; the key
  never transits a proxy.
- The mock-server integration test pattern established for OpenAI is reused
  for every adapter.

## Justification

An OpenAI-compatible-first strategy covers the majority of providers with a
single adapter while keeping the trait boundary that later adapters plug into.
Protocol fixtures plus a local mock server give deterministic tests without
network access, per the testing rules.
