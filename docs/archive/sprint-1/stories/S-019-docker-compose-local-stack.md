---
id: S-019
epic: E-009
title: Docker Compose Local Stack
status: open
priority: high
dependencies: []
---

# S-019: Docker Compose Local Stack

## Purpose

Any developer should be able to go from `git clone` to a running local environment with `docker compose up`. No Homebrew-specific steps, no manual Postgres installation, no "works on my machine" gaps. The Compose file is lightweight — just the infrastructure services that the Rust binary and frontend depend on.

## Scope

- `docker-compose.yml` with Postgres 18 + PostGIS 3.6 and Valkey
- Migrations run automatically on container startup
- `just dev-stack` recipe orchestrates everything (Compose + API + Worker + Frontend)
- `just dev-db` recipe for just the database (for running integration tests)
- Environment variable conventions documented (TEST_DATABASE_URL, etc.)
- `.env.example` with all required variables

## Tickets

- T-019-01: Docker Compose file + migration bootstrap
- T-019-02: Dev stack recipes + environment config
