<script lang="ts">
	import { AppBar, Navigation, Modal } from '@skeletonlabs/skeleton-svelte';
	import { goto } from '$app/navigation';
	import {
		Copy,
		HardDrive,
		History,
		Sliders,
		CircleAlert,
		CircleCheck,
		ChevronsUp,
		FolderOpen,
		Usb,
		Cpu,
		MemoryStick
	} from 'lucide-svelte';
	import { onMount, onDestroy } from 'svelte';
	import { Resource, invoke } from '@tauri-apps/api/core';
	import WebSocket from '@tauri-apps/plugin-websocket';
	import { writable } from 'svelte/store';
	import { deviceStore, type DeviceBase } from '$lib/stores/deviceStore';
	import ProcessModal from '$lib/components/modals/ProcessModal.svelte';
	import { runningProcessesStore } from '$lib/stores/processStore';
	import LogoModal from '$lib/components/modals/LogoModal.svelte';

	// Výchozí hodnoty pro meta
	export let meta: { side: string } = { side: '' };
	export let openState: boolean = false;

	let processModalOpen = false;
	let logoModalOpen = false;

	const navValue = writable('files');
	const handleNavigation = (path: string, value: string) => {
		navValue.set(value);
		goto(path);
	};

	const loading = writable(true);
	let errorMessage: string | null = null;

	const getFormattedDateTime = (): string => {
		const date = new Date();
		const formattedDate = date.toLocaleDateString('cs-CZ', {
			day: 'numeric',
			month: 'numeric'
		});
		const formattedTime = date.toLocaleTimeString('cs-CZ', {
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		});
		return `${formattedDate} ${formattedTime}`;
	};

	let currentDateTime: string = getFormattedDateTime();
	const dateTimeInterval = setInterval(() => {
		currentDateTime = getFormattedDateTime();
	}, 3000);

	let ws: Awaited<ReturnType<typeof WebSocket.connect>> | null = null;

	onMount(async () => {
		try {
			console.log('Zahajuji navazování websocket spojení...');
			const wsAddr = await invoke<string>('start_websocket_server');
			console.log('Websocket server spuštěn na:', wsAddr);

			ws = await WebSocket.connect(wsAddr);
			console.log('Websocket connect() dokončeno, nastavujeme listener...');

			ws.addListener((msg) => {
				console.log('Listener spuštěn, příchozí zpráva:', msg);
				try {
					let jsonStr: string;
					if (typeof msg === 'string') {
						console.log('Zpráva je typu string:', msg);
						jsonStr = msg;
					} else if (msg.data) {
						console.log('Zpráva obsahuje msg.data:', msg.data);
						jsonStr = String(msg.data);
					} else {
						console.error('Neplatný formát zprávy:', msg);
						return;
					}

					const update = JSON.parse(jsonStr);
					console.log('Parsovaný objekt:', update);

					if (update.type === 'ProcessFull') {
						console.log('ProcessFull update:', update);
						runningProcessesStore.update((processes) => {
							const processId = update.id.toString(); // Convert id to string
							const index = processes.findIndex((p) => p.id === processId);
							if (index === -1) {
								// Add new process with id as string
								return [
									...processes,
									{
										...update,
										id: processId,
										out_log: update.out_log || []
									}
								];
							} else {
								// Merge existing process, ensure id is string
								return processes.map((p) =>
									p.id === processId
										? { ...p, ...update, id: processId, out_log: p.out_log || [] }
										: p
								);
							}
						});
					} else if (update.type === 'ProcessProgress') {
						console.log('ProcessProgress update:', update);
						runningProcessesStore.update((processes) => {
							const processId = update.id.toString();
							return processes.map((p) => {
								if (p.id === processId) {
									return {
										...p,
										progress_perc: update.progress_perc ?? p.progress_perc,
										progress_time: update.progress_time ?? p.progress_time,
										speed: update.speed ?? p.speed
									};
								}
								return p;
							});
						});
					} else if (update.type === 'ProcessOutput') {
						console.log('ProcessOutput update:', update);
						runningProcessesStore.update((processes) => {
							const processId = update.id.toString();
							return processes.map((p) => {
								if (p.id === processId) {
									return {
										...p,
										out_log: [...(p.out_log || []), update.output]
									};
								}
								return p;
							});
						});
					} else if (update.type === 'ProcessDone') {
						console.log('ProcessDone update:', update);
						runningProcessesStore.update((processes) => {
							const processId = update.id.toString();
							return processes.map((p) => {
								if (p.id === processId) {
									return {
										...p,
										status: update.status,
										end_datetime: update.end_datetime,
										progress_perc: 100,
										progress_time: 0
									};
								}
								return p;
							});
						});
					} else if (update.type === 'Status') {
						console.log('Status update:', update);
						deviceStore.set(update.data);
					} else {
						console.error('Nečekaný typ zprávy:', update.type);
					}
				} catch (e) {
					console.error('Chyba při parsování zprávy:', e);
				}
			});

			console.log('Listener nastaven, čekám na zprávy...');
		} catch (error) {
			console.error('Chyba při navazování websocket spojení:', error);
		}
	});

	onDestroy(async () => {
		clearInterval(dateTimeInterval);
		if (ws) await ws.disconnect();
	});

	// Deklarace allDisks mimo reaktivní blok
	let allDisks: DeviceBase[] = [];
	$: allDisks = [
		...$deviceStore.usb_devices.filter((d) => d.side === meta.side),
		...$deviceStore.sata_devices.filter((d) => d.side === meta.side)
	];
</script>

<div class="container">
	<Navigation.Rail>
		{#snippet header()}
			<Navigation.Tile href="#" title="Menu">
				<button on:click={() => (logoModalOpen = true)}>
					<img src="/rangers-logo.png" alt="Rangers Logo" class="logo fade-text" />
				</button>
			</Navigation.Tile>
		{/snippet}
		{#snippet tiles()}
			<Navigation.Tile label="Klonování" href="/dashboard/disk_clone">
				<Copy />
			</Navigation.Tile>
			<Navigation.Tile label="Disk Info" href="/dashboard/disk_info">
				<HardDrive />
			</Navigation.Tile>
			<Navigation.Tile label="Správce" href="/dashboard/disk_manager">
				<FolderOpen />
			</Navigation.Tile>
			<Navigation.Tile label="Historie" href="/dashboard">
				<History />
			</Navigation.Tile>
			<Navigation.Tile label="Konfigurace" href="/dashboard/pre_configs">
				<Sliders />
			</Navigation.Tile>
		{/snippet}
	</Navigation.Rail>

	<div class="main-area">
		<header class="header">
			<div class="header-left">
				<span class="header-text">{currentDateTime}</span>
			</div>
			<div class="device-indicators">
				{#if errorMessage}
					<span class="error-message">{errorMessage}</span>
				{:else}
					{#each $deviceStore.usb_devices as device}
						{#if device.side === 'input'}
							<!-- Zobrazení pro vstupní USB zařízení -->
							<span title={`USB Device (Input): ${device.name}`} class="device-indicator">
								<div class="connected-icon " style="width: 20px;">
									<Usb />
								</div>
								<span class="device-name">{device.name}</span>
							</span>
						{:else if device.side === 'output'}
							<!-- Zobrazení pro výstupní USB zařízení -->
							<span title={`USB Device (Output): ${device.name}`} class="device-indicator">
								{#if device.mountpoint}
									<div class="connected-icon" style="width: 20px;">
										<Usb />
									</div>
								{:else}
									<div class="connected-icon" style="width: 20px;">
										<CircleAlert />
									</div>
								{/if}

								<span class="device-name">{device.name}</span>
							</span>
						{/if}
					{/each}

					{#each $deviceStore.sata_devices as device}
						<span title={`SATA Device: ${device.name}`} class="device-indicator">
							<div class="connected-icon" style="width: 20px;">
								<HardDrive />
							</div>
							<span class="device-name">{device.name}</span>
						</span>
					{/each}
				{/if}
			</div>
			<div class="header-right">
				<span class="sysinfo-span">
					<div class="connected-icon" style="width: 20px;">
						<Cpu />
					</div>
					<span class="device-name">{($deviceStore.cpu_usage ?? 0.0).toFixed(1)}%</span>
					<div class="connected-icon" style="width: 20px; margin-left: 5px;">
						<MemoryStick />
					</div>
					<span class="device-name">{($deviceStore.ram_usage ?? 0.0).toFixed(1)}%</span>
				</span>
			</div>
		</header>

		<main class="main-content">
			<slot />
		</main>
	</div>
	<button class="floating-div" on:click={() => (processModalOpen = true)}>
		<div style="width: 10px; margin-right: 30px;"><ChevronsUp /></div>
		Procesy
		<div style="width: 10px; margin-left: 30px;"><ChevronsUp /></div>
	</button>
</div>

<ProcessModal bind:openState={processModalOpen} />
<LogoModal bind:openState={logoModalOpen} />

<style lang="postcss">
	.container {
		display: flex;
		height: 100vh;
		width: 100vw;
	}
	.main-area {
		display: flex;
		flex-direction: column;
		flex: 1;
		overflow: hidden;
	}
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		position: relative;
		height: 40px;
		background-color: var(--color-surface-900);
		padding: 0 2rem;
		box-shadow: 0 1px 40px rgba(212, 22, 60, 0.9);
		width: 100%;
		box-sizing: border-box;
	}
	.sysinfo-span {
		display: flex;
		align-items: center;
	}
	.header-left {
		flex: 1;
		display: flex;
		justify-content: flex-start;
	}
	.device-indicators {
		display: flex;
		gap: 1rem;
		justify-content: center;
		flex: 1;
	}
	.header-right {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: flex-end;
	}
	.header-text {
		font-size: 1rem;
		font-weight: bold;
		color: #d4163c;
	}
	.connected-icon {
		color: #d4163c;
		transition: color 0.3s ease;
	}
	.device-name {
		margin-left: 0.6rem;
		font-size: 0.75rem;
		font-weight: bold;
		color: #736d6c;
	}
	.device-indicator {
		display: flex;
		align-items: center;
	}
	.error-message {
		color: red;
		font-size: 0.875rem;
	}
	.main-content {
		flex: 1;
		padding: 1.5rem;
		overflow-y: auto;
	}
	.logo {
		margin: auto;
		height: 80px;
	}
	.floating-div {
		position: fixed;
		bottom: 0px;
		left: 50%;
		transform: translateX(-50%);
		width: 200px;
		height: 30px;
		background-color: #d4163c;
		font-weight: 1000;
		color: white;
		border: none;
		border-radius: 20px 20px 0 0;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
	}
	.floating-div::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 50%;
		border-top-left-radius: 20px;
		border-top-right-radius: 20px;
	}
</style>
