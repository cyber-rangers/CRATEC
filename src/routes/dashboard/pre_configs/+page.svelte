<script lang="ts">
	import { Popover } from '@skeletonlabs/skeleton-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getContext } from 'svelte';
	import { createToaster } from '@skeletonlabs/skeleton-svelte';
	import { Toaster } from '@skeletonlabs/skeleton-svelte';

	export const toaster = createToaster();

	import DDConfigDetailModal from '$lib/components/modals/DDConfigDetailModal.svelte';
	import EwfConfigDetailModal from '$lib/components/modals/EwfConfigDetailModal.svelte';

	let DDConfigDetailModalOpen = false;
	let EwfConfigDetailModalOpen = false;

	// Přepínač: 0 = ewfacquire, 1 = DD
	let value: number = 0;
	// Očekávaná struktura: { dd: [...], ewf: [...] }
	let configs: any = { dd: [], ewf: [] };
	let popupSettings: Record<number, any> = {};

	// Stav pro detailní modály (původní)
	let showDDModal = false;
	let selectedDDConfig: any = null;
	let showEwfModal = false;
	let selectedEwfConfig: any = null;

	let deletingId: number | null = null;
	let deleteTimeout: any = null;
	let holdProgress: Record<number, number> = {};

	async function loadConfigs() {
		try {
			configs = await invoke('get_all_active_configs');
			console.log('Loaded configs:', configs);
			popupSettings = {};
			configs.ewf.forEach((config: any) => {
				popupSettings[config.id] = {
					event: 'click',
					target: `config-popup-${config.id}`,
					placement: 'bottom'
				};
			});
			configs.dd.forEach((config: any) => {
				popupSettings[config.id] = {
					event: 'click',
					target: `config-popup-${config.id}`,
					placement: 'bottom'
				};
			});
		} catch (error) {
			console.error('Error fetching configs:', error);
		}
	}

	// Otevře novou konfiguraci – podle přepínače naviguje na příslušnou stránku
	function openFormModal() {
		console.log('Otevírám formulář pro vytvoření nové konfigurace');
		if (value === 0) {
			goto('/dashboard/pre_configs/new_ewf_config');
		} else if (value === 1) {
			goto('/dashboard/pre_configs/new_dd_config');
		}
	}

	async function deleteConfig(configId: number) {
		try {
			await invoke('delete_or_deactivate_config', {
				config_id: configId,
				config_type: 'ewf'
			});
			console.log(`Konfigurace ${configId} byla smazána.`);
			toaster.create({
				title: 'Úspěch',
				description: 'Konfigurace byla odstraněna.',
				type: 'success'
			});

			await loadConfigs();
			deletingId = null;
			holdProgress[configId] = 0;
		} catch (error) {
			console.error('Chyba při mazání konfigurace:', error);
			toaster.create({
				title: 'Error',
				description: 'Nepodařilo se odstranit konfiguraci.',
				type: 'error'
			});
		}
	}

	async function deleteDDConfig(configId: number) {
		try {
			await invoke('delete_or_deactivate_config', {
				config_id: configId,
				config_type: 'dd'
			});
			console.log(`DD konfigurace ${configId} byla smazána.`);
			toaster.create({
				title: 'Úspěch',
				description: 'Konfigurace byla odstraněna.',
				type: 'success'
			});

			await loadConfigs();
			deletingId = null;
			holdProgress[configId] = 0;
		} catch (error) {
			console.error('Chyba při mazání DD konfigurace:', error);
			toaster.create({
				title: 'Error',
				description: 'Nepodařilo se odstranit konfiguraci.',
				type: 'error'
			});
		}
	}

	function handleDeleteHoldStart(configId: number, type: 'ewf' | 'dd') {
		deletingId = configId;
		holdProgress[configId] = 0;
		const start = Date.now();
		const duration = 2000;
		function animate() {
			if (deletingId !== configId) return;
			const elapsed = Date.now() - start;
			holdProgress[configId] = Math.min(elapsed / duration, 1);
			if (holdProgress[configId] < 1) {
				requestAnimationFrame(animate);
			} else {
				// Hotovo, smažeme
				if (type === 'ewf') {
					deleteConfig(configId);
				} else {
					deleteDDConfig(configId);
				}
				deletingId = null;
				holdProgress[configId] = 0;
			}
		}
		animate();
		deleteTimeout = setTimeout(() => {}, duration); // pro kompatibilitu s handleDeleteHoldEnd
	}

	function handleDeleteHoldEnd(configId: number) {
		clearTimeout(deleteTimeout);
		if (deletingId === configId && holdProgress[configId] < 1) {
			deletingId = null;
			holdProgress[configId] = 0;
		}
	}

	onMount(async () => {
		await loadConfigs();
	});

	// Funkce pro uzavření detailních modálů (pokud se stále používají)
	function closeDDModal() {
		showDDModal = false;
	}

	function closeEwfModal() {
		showEwfModal = false;
	}
</script>

<div class="container">
	<nav class="btn-group preset-outlined-surface-200-800 flex-col p-2 md:flex-row">
		<!-- Levé tlačítko = ewfacquire -->
		<button
			type="button"
			on:click={() => (value = 0)}
			class="btn {value === 0 ? 'preset-filled' : 'hover:preset-tonal'}"
		>
			ewfacquire
		</button>
		<!-- Pravé tlačítko = DD -->
		<button
			type="button"
			on:click={() => (value = 1)}
			class="btn {value === 1 ? 'preset-filled' : 'hover:preset-tonal'}"
		>
			DD
		</button>
	</nav>
	<button
		on:click={openFormModal}
		type="button"
		class="btn preset-filled-primary-500 flex h-10 w-10 items-center justify-center rounded-full leading-none"
	>
		+
	</button>
</div>

<div class="config-grid">
	<section class="grid grid-cols-2 gap-4 md:grid-cols-4">
		{#if value === 0}
			<!-- Vykreslíme seznam ewfacquire konfigurací -->
			{#each configs.ewf as config (config.id)}
				<Popover
					positioning={{ placement: 'bottom' }}
					triggerBase=""
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="10"
				>
					{#snippet trigger()}
						<div class="grid-box">
							<div class="variant-ringed-primary"></div>
							<div class="centered-container">
								<p class="name">{config.confname}</p>
								<p class="text">ID: {config.id}</p>
								<p class="title">Formát</p>
								<p class="text">{config.ewf_format}</p>
							</div>
						</div>
					{/snippet}
					{#snippet content()}
						<article>
							<button
								class="btn w-full text-left"
								on:click={() => {
									selectedEwfConfig = config;
									EwfConfigDetailModalOpen = true;
								}}
							>
								Info
							</button>
							<button
								class="btn relative flex w-full items-center justify-center overflow-hidden text-left text-red-500"
								on:mousedown={() => handleDeleteHoldStart(config.id, 'ewf')}
								on:touchstart={() => handleDeleteHoldStart(config.id, 'ewf')}
								on:mouseup={() => handleDeleteHoldEnd(config.id)}
								on:mouseleave={() => handleDeleteHoldEnd(config.id)}
								on:touchend={() => handleDeleteHoldEnd(config.id)}
								disabled={!!deletingId && deletingId !== config.id}
							>
								<span
									class="hold-overlay"
									style="width: {Math.round((holdProgress[config.id] || 0) * 100)}%;"
								></span>
								<span class="relative z-10"
									>{deletingId === config.id ? 'Drž pro smazání…' : 'Smazat'}</span
								>
							</button>
						</article>
					{/snippet}
				</Popover>
			{/each}
		{:else if value === 1}
			<!-- Vykreslíme seznam DD konfigurací -->
			{#each configs.dd as config (config.id)}
				<Popover
					positioning={{ placement: 'bottom' }}
					triggerBase=""
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="10"
				>
					{#snippet trigger()}
						<div class="grid-box">
							<div class="variant-ringed-primary"></div>
							<div class="centered-container">
								<p class="name">{config.confname}</p>
								<p class="text">ID: {config.id}</p>
								<p class="title">Block size</p>
								<p class="text">{config.bs} b</p>
							</div>
						</div>
					{/snippet}
					{#snippet content()}
						<article>
							<button
								class="btn w-full text-left"
								on:click={() => {
									selectedDDConfig = config;
									DDConfigDetailModalOpen = true;
								}}
							>
								Info
							</button>
							<button
								class="btn relative flex w-full items-center justify-center overflow-hidden text-left text-red-500"
								on:mousedown={() => handleDeleteHoldStart(config.id, 'dd')}
								on:touchstart={() => handleDeleteHoldStart(config.id, 'dd')}
								on:mouseup={() => handleDeleteHoldEnd(config.id)}
								on:mouseleave={() => handleDeleteHoldEnd(config.id)}
								on:touchend={() => handleDeleteHoldEnd(config.id)}
								disabled={!!deletingId && deletingId !== config.id}
							>
								<span
									class="hold-overlay"
									style="width: {Math.round((holdProgress[config.id] || 0) * 100)}%;"
								></span>
								<span class="relative z-10"
									>{deletingId === config.id ? 'Drž pro smazání…' : 'Smazat'}</span
								>
							</button>
						</article>
					{/snippet}
				</Popover>
			{/each}
		{/if}
	</section>
</div>

<DDConfigDetailModal
	bind:openState={DDConfigDetailModalOpen}
	config={selectedDDConfig}
	on:close={closeDDModal}
/>
<EwfConfigDetailModal
	bind:openState={EwfConfigDetailModalOpen}
	config={selectedEwfConfig}
	on:close={closeEwfModal}
/>
<Toaster {toaster}></Toaster>

<style lang="postcss">
	.container {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
	}

	.config-grid {
		padding-top: 20px;
		width: 50rem;
		margin: auto;
		max-height: 28rem;
		overflow-y: auto;
	}

	.grid-box {
		position: relative;
		width: 8rem;
		height: 8rem;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
	}

	.variant-ringed-primary {
		position: absolute;
		width: 100%;
		height: 100%;
		border: 4px solid var(--color-primary-500);
		border-radius: 0.5rem;
		z-index: 2;
	}

	.centered-container {
		position: absolute;
		width: 100%;
		height: 100%;
		background-color: var(--color-surface-900);
		padding-top: 15px;
		padding-bottom: 5px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		text-align: center;
		border-radius: 28px;
		z-index: 3;
	}

	.centered-container .name {
		font-weight: bold;
		font-size: 1.5rem;
		color: #d4163c;
		padding-bottom: 10px;
	}

	.centered-container .title {
		font-weight: bold;
		font-size: 1rem;
		color: #d4163c;
	}

	.centered-container .text {
		font-size: 1rem;
		color: white;
	}

	.loader {
		border: 2px solid #f3f3f3;
		border-top: 2px solid #d4163c;
		border-radius: 50%;
		width: 1em;
		height: 1em;
		animation: spin 1s linear infinite;
		display: inline-block;
		vertical-align: middle;
	}

	.hold-overlay {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		background: rgba(212, 22, 60, 0.25);
		transition: width 0.1s linear;
		z-index: 1;
		pointer-events: none;
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}
</style>
