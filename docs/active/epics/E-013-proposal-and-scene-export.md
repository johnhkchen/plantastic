---
id: E-013
title: Proposal PDF & Scene Export
status: open
priority: high
sprint: 2
---

## Context

Quoting is the strongest area (40% of budget) but a landscaper can't *send* a quote to a homeowner. PDF export (S.3.3) is the bridge between "cool demo" and "tool that wins contracts." Meanwhile, the 3D viewer exists but shows test scenes — pt-scene is needed to generate glTF from real project data, which also unblocks S.2.4 (★★★), S.4.1, and S.4.3.

Both capabilities double as early system integration checks: if you can export a PDF and a glTF from the same project, the full pipeline (zones → materials → quote → render) is proven.

## Architecture

### PDF Export (BAML + Typst)
```
pt-quote (exact numbers, 3 tiers)
  + project metadata + tenant branding
        │
        ▼
  BAML GenerateProposalNarrative
  (narrative text only — numbers never touch the LLM)
        │
        ▼
  pt-proposal: merge Quote data + ProposalContent
        │
        ▼
  Typst template → Vec<u8> PDF bytes → S3
```

BAML generates human-readable narrative (tier descriptions, zone callouts). All dollar amounts come from pt-quote — the LLM never sees or recalculates prices. Typst renders the branded PDF in-process (no subprocess, no headless Chrome).

**Mocking:** Integration tests use a MockProposalGenerator returning canned ProposalContent. Real LLM calls only in manual smoke tests or the BAML playground.

### Scene Export (pt-scene)
```
zones + material assignments + material textures
        │
        ▼
  pt-scene: generate_scene(project, tier) → SceneOutput
        │
        ▼
  glTF 2.0 binary (.glb) with zone meshes + PBR materials
        │
        ▼
  S3 → presigned URL → Bevy viewer loads via postMessage
```

## Stories

- S-029: BAML Proposal Narrative (schema + mock + API route)
- S-030: Typst PDF Rendering (template + pt-proposal crate)
- S-031: Scene Generation (pt-scene crate + viewer integration)

## Success Criteria

- S.3.3 (Branded PDF export) passes at ★★☆☆☆+ (API route returns PDF bytes)
- S.2.4 advances to ★★★☆☆ (real project data rendered in viewer)
- Integration tests mock all LLM calls (zero token cost)
- `just check` passes
