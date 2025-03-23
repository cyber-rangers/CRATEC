<script lang="ts">
	// Import komponent z SkeletonUI
	import { AppBar, Navigation, Modal } from '@skeletonlabs/skeleton-svelte';
	import { goto } from '$app/navigation';

	// Import ikon z lucide-svelte pro AppBar a Navigation
	import {
		Copy,
		HardDrive,
		History,
		Sliders,
		Settings,
		ChevronsUp,
		Disc3,
		Usb,
		Cpu,
		MemoryStick
	} from 'lucide-svelte';

	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { writable } from 'svelte/store';
	import { deviceStore } from '$lib/stores/deviceStore';

	// Import modálů
	import ProcessModal from '$lib/components/modals/ProcessModal.svelte';
	import LogoModal from '$lib/components/modals/LogoModal.svelte';

	// Stav pro otevření modálů
	let processModalOpen = false;
	let logoModalOpen = false;

	// Navigation – použijeme nový Navigation komponent
	const navValue = writable('files');
	const handleNavigation = (path: string, value: string) => {
		navValue.set(value);
		goto(path);
	};

	// Rozhraní pro stav zařízení
	interface SataDevice {
		interface: string;
	}
	interface UsbDevice {
		interface: string;
	}
	interface DeviceStatus {
		usb_devices: UsbDevice[];
		sata_devices: SataDevice[];
		cpu_usage: number;
		ram_usage: number;
	}

	// Store pro stav zařízení
	const deviceStatus = writable<DeviceStatus>({
		usb_devices: [],
		sata_devices: [],
		cpu_usage: 0,
		ram_usage: 0
	});
	const loading = writable(true);
	let errorMessage: string | null = null;

	// Funkce pro formátování data a času
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
	onDestroy(() => {
		clearInterval(dateTimeInterval);
	});

	let fetchTimeout: ReturnType<typeof setTimeout>;
	let isFetching = false;

	const fetchDeviceStatus = async () => {
		if (isFetching) return;
		isFetching = true;
		try {
			loading.set(true);
			const status: DeviceStatus = await invoke<DeviceStatus>('get_device_status');
			console.log(status);
			deviceStore.set(status);
			deviceStatus.set(status);
			errorMessage = null;
		} catch (error) {
			console.error('Nepodařilo se získat stav zařízení:', error);
			errorMessage = 'Nepodařilo se získat stav zařízení';
		} finally {
			loading.set(false);
			isFetching = false;
		}
	};

	onMount(() => {
		fetchDeviceStatus();
		const statusInterval = setInterval(() => {
			clearTimeout(fetchTimeout);
			fetchTimeout = setTimeout(fetchDeviceStatus, 500);
		}, 3000);
		return () => {
			clearInterval(statusInterval);
			clearTimeout(fetchTimeout);
		};
	});
</script>

<div class="container">
	<Navigation.Rail>
		{#snippet header()}
			<!-- Kliknutím na logo se otevře LogoModal -->
			<Navigation.Tile href="#" title="Menu">
				<button onclick={() => (logoModalOpen = true)}>
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
			<Navigation.Tile label="Formátování" href="/dashboard">
				<Disc3 />
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
					{#each $deviceStatus.usb_devices as device}
						<span title={`USB Device Interface: ${device.interface}`} class="device-indicator">
							<div class="connected-icon" style="width: 20px;">
								<Usb />
							</div>
							<span class="device-name">{device.interface}</span>
						</span>
					{/each}

					{#each $deviceStatus.sata_devices as device}
						<span title={`SATA Device Interface: ${device.interface}`} class="device-indicator">
							<div class="connected-icon" style="width: 20px;">
								<HardDrive />
							</div>
							<span class="device-name">{device.interface}</span>
						</span>
					{/each}
				{/if}
			</div>

			<div class="header-right">
				<span class="sysinfo-span">
					<div class="connected-icon" style="width: 20px;">
						<Cpu />
					</div>
					<span class="device-name">{$deviceStatus.cpu_usage.toFixed(1)}%</span>
					<div class="connected-icon" style="width: 20px; margin-left: 5px;">
						<MemoryStick />
					</div>
					<span class="device-name">{$deviceStatus.ram_usage.toFixed(1)}%</span>
				</span>
			</div>
		</header>

		<main class="main-content">
			<slot />
		</main>
	</div>
	<!-- Kliknutím na tlačítko Procesy se otevře ProcessModal -->
	<button class="floating-div" onclick={() => (processModalOpen = true)}>
		<div style="width: 10px; margin-right: 30px;"><ChevronsUp /></div>
		Procesy
		<div style="width: 10px;margin-left: 30px;"><ChevronsUp /></div>
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
