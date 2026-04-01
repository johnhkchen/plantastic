<script lang="ts">
	import { apiFetch } from '$lib/api';
	import type { Material } from '$lib/stores/project.svelte';
	import { onMount } from 'svelte';
	import CatalogFilter from '$lib/components/catalog/CatalogFilter.svelte';
	import { formatPrice, formatUnit } from '$lib/utils/format';

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

	const categoryColors: Record<string, string> = {
		hardscape: 'bg-stone-100 text-stone-700',
		softscape: 'bg-green-100 text-green-700',
		edging: 'bg-amber-100 text-amber-700',
		fill: 'bg-orange-100 text-orange-700'
	};
</script>

<div class="mx-auto max-w-4xl">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="font-display text-2xl font-bold text-gray-900">Material Catalog</h1>
		<button
			onclick={openCreateModal}
			class="rounded-md bg-brand-primary px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-brand-secondary transition-colors"
		>
			Add Material
		</button>
	</div>

	{#if !loading && materials.length > 0}
		<CatalogFilter {materials} onfilter={(filtered) => (filteredMaterials = filtered)} />
	{/if}

	{#if error}
		<div
			class="mb-4 rounded-md bg-red-50 border border-red-200 p-4 flex items-center justify-between"
		>
			<p class="text-sm text-red-700">{error}</p>
			<button
				onclick={loadMaterials}
				class="text-sm font-medium text-red-700 hover:text-red-800 underline"
			>
				Retry
			</button>
		</div>
	{/if}

	{#if loading}
		<div class="space-y-3">
			{#each [1, 2, 3] as n (n)}
				<div class="rounded-lg border border-gray-200 bg-white p-5 animate-pulse">
					<div class="h-5 bg-gray-200 rounded w-1/4 mb-3"></div>
					<div class="h-4 bg-gray-100 rounded w-1/3"></div>
				</div>
			{/each}
		</div>
	{:else if materials.length === 0 && !error}
		<div class="rounded-lg border border-gray-200 bg-white p-12 text-center">
			<p class="text-gray-500">
				No materials in the catalog. Add your first material to get started.
			</p>
		</div>
	{:else if filteredMaterials.length === 0 && materials.length > 0}
		<div class="rounded-lg border border-gray-200 bg-white p-12 text-center">
			<p class="text-gray-500">No materials match your search.</p>
		</div>
	{:else}
		<div class="overflow-hidden rounded-lg border border-gray-200 bg-white">
			<table class="min-w-full divide-y divide-gray-200">
				<thead class="bg-gray-50">
					<tr>
						<th
							class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
							>Name</th
						>
						<th
							class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
							>Category</th
						>
						<th
							class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
							>Unit</th
						>
						<th
							class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
							>Price</th
						>
						<th
							class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
						></th>
					</tr>
				</thead>
				<tbody class="divide-y divide-gray-100">
					{#each filteredMaterials as mat (mat.id)}
						<tr class="hover:bg-gray-50 transition-colors">
							<td class="px-4 py-3 text-sm font-medium text-gray-900">{mat.name}</td>
							<td class="px-4 py-3">
								<span
									class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium {categoryColors[
										mat.category
									] ?? 'bg-gray-100 text-gray-700'}"
								>
									{mat.category}
								</span>
							</td>
							<td class="px-4 py-3 text-sm text-gray-500">{formatUnit(mat.unit)}</td>
							<td class="px-4 py-3 text-sm text-gray-900 text-right font-mono">
								{formatPrice(mat.price_per_unit)}/{formatUnit(mat.unit)}
							</td>
							<td class="px-4 py-3 text-right">
								<button
									onclick={() => openEditModal(mat)}
									class="text-sm text-brand-primary hover:text-brand-secondary mr-3"
								>
									Edit
								</button>
								<button
									onclick={() => deleteMaterial(mat)}
									class="text-sm text-red-600 hover:text-red-700"
								>
									Delete
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<!-- Create / Edit Material Modal -->
{#if showModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
		<div class="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
			<h2 class="text-lg font-semibold text-gray-900 mb-4">
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
						<label for="mat_name" class="block text-sm font-medium text-gray-700">Name</label>
						<input
							id="mat_name"
							type="text"
							required
							bind:value={formName}
							placeholder="Flagstone Pavers"
							class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
						/>
					</div>
					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="mat_category" class="block text-sm font-medium text-gray-700"
								>Category</label
							>
							<select
								id="mat_category"
								bind:value={formCategory}
								class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
							>
								<option value="hardscape">Hardscape</option>
								<option value="softscape">Softscape</option>
								<option value="edging">Edging</option>
								<option value="fill">Fill</option>
							</select>
						</div>
						<div>
							<label for="mat_unit" class="block text-sm font-medium text-gray-700">Unit</label>
							<select
								id="mat_unit"
								bind:value={formUnit}
								class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
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
							<label for="mat_price" class="block text-sm font-medium text-gray-700"
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
								class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
							/>
						</div>
						<div>
							<label for="mat_depth" class="block text-sm font-medium text-gray-700"
								>Depth (inches)</label
							>
							<input
								id="mat_depth"
								type="number"
								step="0.5"
								min="0"
								bind:value={formDepth}
								placeholder="2.0"
								class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
							/>
						</div>
					</div>
					<div>
						<label for="mat_sku" class="block text-sm font-medium text-gray-700">Supplier SKU</label
						>
						<input
							id="mat_sku"
							type="text"
							bind:value={formSku}
							placeholder="FLG-001"
							class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
						/>
					</div>
				</div>
				<div class="mt-6 flex justify-end gap-3">
					<button
						type="button"
						onclick={() => (showModal = false)}
						class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={saving}
						class="rounded-md bg-brand-primary px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-brand-secondary disabled:opacity-50 transition-colors"
					>
						{saving ? 'Saving...' : editingMaterial ? 'Save Changes' : 'Add Material'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
