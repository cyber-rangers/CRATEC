<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { LockKeyhole, Power,  FileCog, X } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import LockScreenDrawer from '$lib/components/modals/LockScreenDrawer.svelte';
	import SystemLogModal from '$lib/components/modals/SystemLogModal.svelte';
	import { onMount } from 'svelte';
	import { getVersion, getName } from '@tauri-apps/api/app';

	let appVersion = '';
	let appName = '';

	let ewfacquireVersion = '';
	let dcflddVersion = '';
	let buildDate = '';

	// Funkce pro formátování ISO data na "14:00 17.10.2000"
	function formatBuildDate(dateStr: string): string {
		if (!dateStr) return '';
		const d = new Date(dateStr);
		const pad = (n: number) => n.toString().padStart(2, '0');
		return `${pad(d.getHours())}:${pad(d.getMinutes())} ${pad(d.getDate())}.${pad(d.getMonth() + 1)}.${d.getFullYear()}`;
	}

	onMount(async () => {
		appVersion = await getVersion();
		appName = await getName();

		try {
			const versions = await invoke<{
				ewfacquire: string;
				dcfldd: string;
				build_date: string;
			}>('get_program_versions');
			ewfacquireVersion = versions.ewfacquire;
			dcflddVersion = versions.dcfldd;
			buildDate = versions.build_date;
		} catch (e) {
			console.error('Chyba při získávání verzí programů:', e);
		}
	});

	export let openState: boolean = false;
	let lockDrawerVisible = false;
	let systemLogOpen = false;

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
	function openSystemLog() {
		systemLogOpen = true;
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

		<!-- Info section: přehledněji -->
		<div class="mb-4 grid grid-cols-2 gap-4 text-sm">
			<div>
				<p><b>Autor:</b> <code class="code">Martin Sladký</code></p>
				<p><b>Verze aplikace:</b> <code class="code">{appVersion}</code></p>
				<p><b>Čas sestavení:</b> <code class="code">{formatBuildDate(buildDate)}</code></p>
			</div>
			<div class="text-right">
				<p><b>ewfacquire:</b> <code class="code">{ewfacquireVersion.replace(/^ewfacquire\s*/i, '').trim()}</code></p>
				<p><b>dcfldd:</b> <code class="code">{dcflddVersion.replace(/^dcfldd\s*\(dcfldd\)\s*/i, '').trim()}</code></p>
			</div>
		</div>

		<!-- Tlačítka vystředěná -->
		<div class="w-full flex justify-center items-center gap-8 mt-8">
			<button
				type="button"
				class="btn preset-tonal flex flex-col items-center rounded-lg h-32 w-32"
				style="background-color: var(--color-surface-700)"
				on:click={handleLockClick}
			>
				<LockKeyhole class="h-6 w-6" />
				<span class="mt-2 text-sm">Uzamknout</span>
			</button>
			<button
				type="button"
				class="btn preset-tonal flex flex-col items-center rounded-lg h-32 w-32"
				style="background-color: var(--color-surface-700)"
				on:click={shutdownSystem}
			>
				<Power class="h-6 w-6" />
				<span class="mt-2 text-sm">Vypnutí</span>
			</button>
			<button
				type="button"
				class="btn preset-tonal flex flex-col items-center rounded-lg h-32 w-32"
				style="background-color: var(--color-surface-700)"
				on:click={openSystemLog}
			>
				<FileCog class="h-6 w-6" />
				<span class="mt-2 text-sm">System Log</span>
			</button>
		</div>
	{/snippet}
</Modal>

<SystemLogModal bind:openState={systemLogOpen} />

<LockScreenDrawer bind:isOpen={lockDrawerVisible} />

<style>
/* Pro jistotu, aby tlačítka byla opravdu vystředěná */
.w-full.flex.justify-center.items-center {
	margin-left: auto;
	margin-right: auto;
}
</style>
