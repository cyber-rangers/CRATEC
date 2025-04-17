<script lang="ts">
	import { onMount } from 'svelte';
	import { writable } from 'svelte/store';
	import { Usb, HardDrive } from 'lucide-svelte';
	import { deviceStore } from '$lib/stores/deviceStore';
	import { invoke } from '@tauri-apps/api/core';
	import type { DeviceBase } from '$lib/stores/deviceStore';

	// Rust funkce vrací: { name: string, file_type: "folder"|"file" }
	interface FileItem {
		name: string;
		file_type: string;
	}

	// Store pro vybraný disk
	const selectedDisk = writable<any>(null);

	// Store pro soubory z vybraného diskového mountpointu
	const fileItems = writable<FileItem[]>([]);

	let disks: DeviceBase[] = [];

	// Variables to store screen resolution
	let screenWidth = 0;
	let screenHeight = 0;

	onMount(() => {
		// Get current screen resolution on mount
		screenWidth = window.innerWidth;
		screenHeight = window.innerHeight;
		console.log('Screen Resolution:', screenWidth, screenHeight);
	});

	// Předplatíme deviceStore a vezmeme jen zařízení s definovaným mountpointem
	const unsubscribe = deviceStore.subscribe((value) => {
		disks = [...value.usb_devices, ...value.sata_devices].filter((d) => d.mountpoint);
	});

	/**
	 * Po kliknutí na disk:
	 * 1) nastavíme vybraný disk
	 * 2) zavoláme Rust funkci get_directory_contents
	 * 3) uložíme výsledek do fileItems
	 */
	async function selectDisk(disk: any) {
		selectedDisk.set(disk);

		try {
			const result = await invoke<FileItem[]>('get_directory_contents', {
				mountpoint: disk.mountpoint
			});
			fileItems.set(result);
		} catch (err) {
			console.error('Failed to get directory contents:', err);
			fileItems.set([]);
		}
	}

	function goBack() {
		selectedDisk.set(null);
		fileItems.set([]);
	}
</script>

<!-- Display current resolution -->
<div class="resolution-info">Resolution: {screenWidth}x{screenHeight}px</div>

{#if $selectedDisk === null}
	<!-- Zobrazení dostupných disků -->
	<div class="disk-container">
		{#each disks as disk (disk.interface)}
			<div class="disk-box" on:click={() => selectDisk(disk)}>
				<div class="icon">
					{#if disk.type === 'usb'}
						<Usb size="48" />
					{:else if disk.type === 'sata'}
						<HardDrive size="48" />
					{/if}
				</div>
				<div class="disk-info">
					<div class="disk-name">{disk.name || disk.interface}</div>
					<div class="mountpoint">{disk.mountpoint}</div>
				</div>
			</div>
		{/each}
	</div>
{:else}
	<!-- Zobrazení obsahu vybraného disku -->
	<div class="filemanager-view">
		<button on:click={goBack}>← Back to Disks</button>
		<h2>Files in {$selectedDisk.mountpoint}</h2>
		<ul>
			{#each $fileItems as item}
				<li class={item.file_type}>
					{#if item.file_type === 'folder'}
						<strong>📁 {item.name}</strong>
					{:else}
						<span>📄 {item.name}</span>
					{/if}
				</li>
			{/each}
		</ul>
	</div>
{/if}

<style>
	/* Kontejner s boxy pro disky */
	.disk-container {
		display: flex;
		flex-wrap: wrap;
		gap: 1rem;
		padding: 1rem;
	}

	.disk-box {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		background: var(--color-surface-900);
		border: 3px solid var(--color-primary-600);
		border-radius: 8px;
		padding: 1rem;
		cursor: pointer;
		width: 150px;
		height: 120px;
		text-align: center;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		transition: transform 0.2s;
	}

	.disk-box:hover {
		transform: translateY(-5px);
	}

	.disk-info {
		margin-top: 0.5rem;
	}

	.disk-name {
		font-weight: bold;
		margin-bottom: 0.5rem;
	}

	.filemanager-view {
		padding: 1rem;
	}

	.filemanager-view button {
		margin-bottom: 1rem;
	}

	.filemanager-view ul {
		list-style: none;
		padding: 0;
	}

	.filemanager-view li {
		margin: 0.5rem 0;
		padding: 0.5rem;
		border: 1px solid #eee;
		border-radius: 4px;
	}

	.resolution-info {
		position: absolute;
		top: 10px;
		left: 50%;
		transform: translateX(-50%);
		background-color: rgba(0, 0, 0, 0.7);
		color: white;
		padding: 5px 8px;
		border-radius: 4px;
		font-size: 12px;
		z-index: 100;
	}
</style>
