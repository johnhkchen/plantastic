<script lang="ts">
	import { apiFetch } from '$lib/api';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
	import type { Material } from '$lib/stores/project.svelte';
	import { onMount } from 'svelte';
	import CatalogFilter from '$lib/components/catalog/CatalogFilter.svelte';
	import { formatPrice, formatUnit } from '$lib/utils/format';
	import { CATEGORY_BADGE_COLORS } from '$lib/styles/color-maps';

	let materials = $state<Material[]>([]);
	let filteredMaterials = $state<Material[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Modal state
	let showModal = $state(false);
	let editingMaterial = $state<Material | null>(null);
	let saving = $state(false);

	// Form fields
	let formName = $state('');
	let formCategory = $state<Material['category']>('hardscape');
	let formUnit = $state<Material['unit']>('sq_ft');
	let formPrice = $state('');
	let formDepth = $state('');
	let formSku = $state('');

	onMount(() => {
		loadMaterials();
	});

	async function loadMaterials() {
		loading = true;
		error = null;
		try {
			materials = await apiFetch<Material[]>('/materials');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load materials';
		} finally {
			loading = false;
		}
	}

	function openCreateModal() {
		editingMaterial = null;
		formName = '';
		formCategory = 'hardscape';
		formUnit = 'sq_ft';
		formPrice = '';
		formDepth = '';
		formSku = '';
		showModal = true;
	}

	function openEditModal(mat: Material) {
		editingMaterial = mat;
		formName = mat.name;
		formCategory = mat.category;
		formUnit = mat.unit;
		formPrice = mat.price_per_unit;
		formDepth = mat.depth_inches != null ? String(mat.depth_inches) : '';
		formSku = mat.supplier_sku ?? '';
		showModal = true;
	}

	function buildMaterialBody() {
		return {
			name: formName,
			category: formCategory,
			unit: formUnit,
			price_per_unit: formPrice,
			depth_inches: formDepth ? parseFloat(formDepth) : null,
			extrusion: editingMaterial?.extrusion ?? { type: 'fills', flush: true },
			texture_key: editingMaterial?.texture_key ?? null,
			photo_key: editingMaterial?.photo_key ?? null,
			supplier_sku: formSku || null
		};
	}

	async function saveMaterial() {
		saving = true;
		error = null;
		try {
			if (editingMaterial) {
				await apiFetch(`/materials/${editingMaterial.id}`, {
					method: 'PATCH',
					body: buildMaterialBody()
				});
				// Update in local list
				materials = materials.map((m) =>
					m.id === editingMaterial!.id
						? { ...m, ...buildMaterialBody(), updated_at: new Date().toISOString() }
						: m
				) as Material[];
			} else {
				const created = await apiFetch<Material>('/materials', {
					method: 'POST',
					body: buildMaterialBody()
				});
				materials = [...materials, created];
			}
			showModal = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save material';
		} finally {
			saving = false;
		}
	}

	async function deleteMaterial(mat: Material) {
		if (!confirm(`Delete "${mat.name}"? This cannot be undone.`)) return;
		try {
			await apiFetch(`/materials/${mat.id}`, { method: 'DELETE' });
			materials = materials.filter((m) => m.id !== mat.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete material';
		}
	}
</script>

<div class="mx-auto max-w-4xl">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="font-display text-2xl font-bold text-text">Material Catalog</h1>
		<button
			onclick={openCreateModal}
			class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-text-invert shadow-sm hover:bg-primary-light transition-colors"
		>
			Add Material
		</button>
	</div>

	{#if !loading && materials.length > 0}
		<CatalogFilter {materials} onfilter={(filtered) => (filteredMaterials = filtered)} />
	{/if}

	{#if error}
		<ErrorBanner message={error} onretry={loadMaterials} />
	{/if}

	{#if loading}
		<LoadingSkeleton variant="card" />
	{:else if materials.length === 0 && !error}
		<EmptyState icon="🧱" message="No materials in your catalog" submessage="Add materials before assigning them to zones">
			<button
				onclick={openCreateModal}
				class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-text-invert shadow-sm hover:bg-primary-light transition-colors"
			>
				Add Material
			</button>
		</EmptyState>
	{:else if filteredMaterials.length === 0 && materials.length > 0}
		<div class="rounded-lg border border-border bg-surface p-12 text-center">
			<p class="text-text-secondary">No materials match your search.</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each filteredMaterials as mat (mat.id)}
				<div class="rounded-lg border border-border bg-surface p-4 flex flex-col gap-3 hover:shadow-sm transition-shadow">
					<div class="flex items-start justify-between gap-2">
						<h3 class="text-sm font-medium text-text leading-tight">{mat.name}</h3>
						<span
							class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium flex-shrink-0 {CATEGORY_BADGE_COLORS[
								mat.category
							] ?? 'bg-surface-hover text-text-secondary'}"
						>
							{mat.category}
						</span>
					</div>
					<div class="text-sm text-text font-mono">
						{formatPrice(mat.price_per_unit)}/{formatUnit(mat.unit)}
					</div>
					<div class="flex items-center gap-2 mt-auto pt-2 border-t border-border-light">
						<button
							onclick={() => openEditModal(mat)}
							class="min-h-[44px] flex-1 rounded-md border border-border text-sm font-medium text-primary hover:bg-surface-alt transition-colors"
						>
							Edit
						</button>
						<button
							onclick={() => deleteMaterial(mat)}
							class="min-h-[44px] flex-1 rounded-md border border-border text-sm font-medium text-error hover:bg-error-surface transition-colors"
						>
							Delete
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Create / Edit Material Modal -->
{#if showModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
		<div class="w-full max-w-md rounded-lg bg-surface p-6 shadow-xl">
			<h2 class="text-lg font-semibold text-text mb-4">
				{editingMaterial ? 'Edit Material' : 'Add Material'}
			</h2>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					saveMaterial();
				}}
			>
				<div class="space-y-4">
					<div>
						<label for="mat_name" class="block text-sm font-medium text-text-secondary">Name</label>
						<input
							id="mat_name"
							type="text"
							required
							bind:value={formName}
							placeholder="Flagstone Pavers"
							class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
						/>
					</div>
					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="mat_category" class="block text-sm font-medium text-text-secondary"
								>Category</label
							>
							<select
								id="mat_category"
								bind:value={formCategory}
								class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
							>
								<option value="hardscape">Hardscape</option>
								<option value="softscape">Softscape</option>
								<option value="edging">Edging</option>
								<option value="fill">Fill</option>
							</select>
						</div>
						<div>
							<label for="mat_unit" class="block text-sm font-medium text-text-secondary">Unit</label>
							<select
								id="mat_unit"
								bind:value={formUnit}
								class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
							>
								<option value="sq_ft">sq ft</option>
								<option value="cu_yd">cu yd</option>
								<option value="linear_ft">linear ft</option>
								<option value="each">each</option>
							</select>
						</div>
					</div>
					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="mat_price" class="block text-sm font-medium text-text-secondary"
								>Price per Unit ($)</label
							>
							<input
								id="mat_price"
								type="number"
								step="0.01"
								min="0"
								required
								bind:value={formPrice}
								placeholder="8.50"
								class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
							/>
						</div>
						<div>
							<label for="mat_depth" class="block text-sm font-medium text-text-secondary"
								>Depth (inches)</label
							>
							<input
								id="mat_depth"
								type="number"
								step="0.5"
								min="0"
								bind:value={formDepth}
								placeholder="2.0"
								class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
							/>
						</div>
					</div>
					<div>
						<label for="mat_sku" class="block text-sm font-medium text-text-secondary">Supplier SKU</label
						>
						<input
							id="mat_sku"
							type="text"
							bind:value={formSku}
							placeholder="FLG-001"
							class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
						/>
					</div>
				</div>
				<div class="mt-6 flex justify-end gap-3">
					<button
						type="button"
						onclick={() => (showModal = false)}
						class="rounded-md border border-border-strong px-4 py-2 text-sm font-medium text-text-secondary hover:bg-surface-alt"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={saving}
						class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-text-invert shadow-sm hover:bg-primary-light disabled:opacity-50 transition-colors"
					>
						{saving ? 'Saving...' : editingMaterial ? 'Save Changes' : 'Add Material'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
