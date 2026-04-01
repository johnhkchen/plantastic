# T-012-01 Research — Catalog CRUD Page

## What exists

### Frontend catalog page
`web/src/routes/(app)/catalog/+page.svelte` — fully implemented by T-006-01:
- Lists materials in a table: name, category (badge), unit, price/unit
- Add material: button → modal with form (name, category, unit, price, depth, SKU)
- Edit material: row Edit button → modal pre-filled with existing data
- Delete material: row Delete button → `confirm()` dialog then DELETE request
- Loading state: skeleton placeholders
- Error state: red banner with retry button
- Empty state: "No materials" prompt

All CRUD calls use `apiFetch()` from `web/src/lib/api/client.ts`:
- `GET /materials` → load list on mount
- `POST /materials` → create, append returned material to local list
- `PATCH /materials/${id}` → update, merge into local list
- `DELETE /materials/${id}` → remove from local list

### Backend API routes
`crates/plantastic-api/src/routes/materials.rs` — delivered by T-004-02:
- `GET /materials` → list_materials (tenant-scoped via X-Tenant-Id header)
- `POST /materials` → create_material (returns 201 + MaterialResponse)
- `PATCH /materials/{id}` → update_material (returns 204, tenant ownership check)
- `DELETE /materials/{id}` → delete_material (returns 204, tenant ownership check)

Request body: `CreateMaterialRequest { name, category, unit, price_per_unit, depth_inches?, extrusion, texture_key?, photo_key?, supplier_sku? }`
Response: `MaterialResponse { id, tenant_id, name, category, unit, price_per_unit, depth_inches, extrusion, texture_key, photo_key, supplier_sku, created_at, updated_at }`

### Domain model
`crates/pt-materials/src/types.rs`:
- `Material` struct with MaterialId, name, category, unit, price_per_unit (Decimal), depth_inches, texture_ref, photo_ref, supplier_sku, extrusion
- `MaterialCategory` enum: Hardscape, Softscape, Edging, Fill (serde: snake_case)
- `Unit` enum: SqFt, CuYd, LinearFt, Each (serde: snake_case)
- `ExtrusionBehavior` enum: SitsOnTop{height_inches}, Fills{flush}, BuildsUp{height_inches} (serde: internally tagged, snake_case)
- Builder pattern via `builder.rs`
- 8 unit tests covering serde round-trips

### TypeScript types
`web/src/lib/stores/project.svelte.ts`:
```typescript
interface Material {
  id: string; tenant_id: string; name: string;
  category: 'hardscape' | 'softscape' | 'edging' | 'fill';
  unit: 'sq_ft' | 'cu_yd' | 'linear_ft' | 'each';
  price_per_unit: string; depth_inches: number | null;
  extrusion: unknown; texture_key: string | null;
  photo_key: string | null; supplier_sku: string | null;
  created_at: string; updated_at: string;
}
```

### Repository layer
`crates/pt-repo/src/material.rs`:
- `list_by_tenant(pool, tenant_id)` → `Vec<MaterialRow>`
- `create(pool, input)` → `Uuid`
- `update(pool, id, input)` → `()`
- `delete(pool, id)` → `()`

### Database schema
`migrations/004-create-materials.sql`: materials table with all fields, tenant_id FK, category/unit CHECK constraints, extrusion JSONB.

### Scenario status
`tests/scenarios/src/suites/design.rs`: S.2.2 is `NotImplemented`
`tests/scenarios/src/progress.rs`: "pt-materials: catalog model + tenant layering" milestone has `delivered_by: None`

## Bug found

`web/src/routes/(app)/catalog/+page.svelte` line 68:
```js
extrusion: editingMaterial?.extrusion ?? { type: 'Fills', flush: true }
```
The Rust enum uses `#[serde(tag = "type", rename_all = "snake_case")]`, so the tag value must be `"fills"` (lowercase), not `"Fills"` (PascalCase). Creating a new material from the frontend would send `{"type":"Fills","flush":true}` which Rust serde would reject as an unknown variant. The same pattern applies to `SitsOnTop` → `sits_on_top` and `BuildsUp` → `builds_up`.

## Gap analysis

| Acceptance Criterion | Status | Notes |
|---|---|---|
| /catalog route | Done | T-006-01 delivered |
| List materials | Done | Table with name, category, unit, price |
| Add material | Done* | Modal form; extrusion default has casing bug |
| Edit material | Done | Click Edit → modal pre-filled |
| Delete material | Done | confirm() → DELETE |
| Calls API routes | Done | apiFetch CRUD |
| Empty state | Done | "No materials" prompt |
| S.2.2 registered | Partial | Registered but NotImplemented; needs test body |

## Dependencies
- T-006-01 (dashboard-catalog-integration): Done — delivered the page
- T-004-02 (CRUD routes): Done — delivered the API
- T-003-02 (repository layer): Done — delivered MaterialRepo

## What this ticket must deliver
1. Fix the extrusion casing bug in the catalog page
2. Implement S.2.2 scenario test (move from NotImplemented → OneStar)
3. Claim the pt-materials milestone in progress.rs
