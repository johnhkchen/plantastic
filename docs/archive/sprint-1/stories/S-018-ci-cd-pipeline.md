---
id: S-018
epic: E-008
title: CI/CD Pipeline
status: open
priority: high
dependencies:
  - S-017
---

# S-018: CI/CD Pipeline

## Purpose

Automate quality gates and deployment. Every PR runs `just check` and `just scenarios`. Merge to main triggers deployment of all three components (Lambda, CF Pages, CF Worker). Smoke tests verify the deployment is alive.

## Scope

- GitHub Actions workflow for PR checks
- GitHub Actions workflow for deploy on merge
- CF Pages deployment of SvelteKit build
- CF Worker deployment with Lambda URL
- Domain setup: get-plantastic.com on Cloudflare, staging subdomain
- Smoke test script: health check + create/fetch/delete project round-trip
- `just deploy` recipe wired to real commands

## Tickets

- T-018-01: GitHub Actions CI (check + scenarios on PR)
- T-018-02: Deploy pipeline + domain + smoke tests
