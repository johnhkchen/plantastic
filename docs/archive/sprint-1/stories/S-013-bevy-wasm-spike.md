---
id: S-013
epic: E-006
title: Bevy WASM Spike
status: open
priority: critical
---

# S-013: Bevy WASM Spike

## Purpose

Prove the Bevy→WASM→browser pipeline works before we build anything on top of it. This is the highest-risk unknown in the frontend stack. Failure here means we need an alternative (three.js, Babylon.js) and that changes the architecture significantly.

## Scope

- Bevy app that loads a glTF file and renders it with PBR materials
- Compiles to WASM via wasm-pack or trunk
- Embedded in SvelteKit page via iframe
- Message passing between SvelteKit host and Bevy iframe (postMessage)
- Performance benchmarking: binary size, load time, frame rate
- Browser compatibility: Chrome, Firefox, Safari desktop, iPad Safari

## Tickets

- T-013-01: Bevy WASM compile + glTF loading
- T-013-02: SvelteKit iframe embedding + message bridge
