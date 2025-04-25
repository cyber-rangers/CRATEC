<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { LockKeyhole, Power, RefreshCw, X } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import LockScreenDrawer from '$lib/components/modals/LockScreenDrawer.svelte';
	import { onMount } from 'svelte';
	import { getVersion, getName } from '@tauri-apps/api/app';

	let appVersion = '';
	let appName = '';

	onMount(async () => {
		appVersion = await getVersion();
		appName = await getName();
	});

	export let openState: boolean = false;
	let lockDrawerVisible = false;

	function modalClose() {
		openState = false;
	}
	function handleLockClick() {
		lockDrawerVisible = true;
	}
	async function shutdownSystem() {
		try {
			await invoke('shutdown_system');
		} catch (e) {
			console.error('Chyba při vypnutí systému:', e);
		}
	}
	async function restartSystem() {
		try {
			await invoke('restart_system');
		} catch (e) {
			console.error('Chyba při restartu systému:', e);
		}
	}
</script>

<Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase=""
	contentBase="card bg-surface-100-900 p-4 pt-12 space-y-6 shadow-xl relative"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		<!-- Close button top‑right inside modal -->
		<button
			type="button"
			class="btn variant-filled-surface-500 absolute top-4 right-4 z-50 rounded-full p-1"
			on:click={modalClose}
			aria-label="Zavřít"
		>
			<X size={20} />
		</button>

		<!-- Info section: two columns with github under build date -->
		<div class="mb-4 grid grid-cols-2 gap-4 text-sm">
			<div>
				<p>Autor: Martin Sladký</p>
				<p>Verze: {appVersion}</p>
			</div>
			<div class="text-right">
				<p>Build date: 17.10.2000</p>
				<p>
					github: <a href="https://example.com" target="_blank" class="underline">example.com</a>
				</p>
			</div>
		</div>

		<article class="flex h-[23vh] w-[40vw] items-center justify-around gap-4">
			<button
				type="button"
				class="btn preset-tonal flex h-32 w-32 flex-col items-center rounded-lg"
				style="background-color: var(--color-surface-700)"
				on:click={handleLockClick}
			>
				<LockKeyhole class="h-6 w-6" />
				<span class="mt-2 text-sm">Uzamknout</span>
			</button>
			<button
				type="button"
				class="btn preset-tonal flex h-32 w-32 flex-col items-center rounded-lg"
				style="background-color: var(--color-surface-700)"
				on:click={shutdownSystem}
			>
				<Power class="h-6 w-6" />
				<span class="mt-2 text-sm">Vypnutí</span>
			</button>
			<button
				type="button"
				class="btn preset-tonal flex h-32 w-32 flex-col items-center rounded-lg"
				style="background-color: var(--color-surface-700)"
				on:click={restartSystem}
			>
				<RefreshCw class="h-6 w-6" />
				<span class="mt-2 text-sm">Restart</span>
			</button>
		</article>
	{/snippet}
</Modal>

<LockScreenDrawer bind:isOpen={lockDrawerVisible} />
