<script lang="ts">
	import { AppRail, AppRailTile, AppRailAnchor, getModalStore } from '@skeletonlabs/skeleton';
	import type { ModalSettings } from '@skeletonlabs/skeleton';
	import { goto } from '$app/navigation';
	import {
		FaHdd,
		FaCog,
		FaHistory,
		FaBroom,
		FaCloudscale,
		FaUsb,
		FaCopy,
		FaAngleDoubleUp,
		FaSlidersH
	} from 'svelte-icons/fa';
	import { onDestroy, onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { writable } from 'svelte/store';
	import { deviceStore } from '$lib/stores/deviceStore';

	const modalStore = getModalStore();

	const modal: ModalSettings = {
		type: 'alert',
		// Data
		title: 'Zde budou aktivní procesy co běží',
		body: '',
		image: ''
	};

	let currentTile = 0;

	const handleNavigation = (path: string, tileValue: number) => {
		currentTile = tileValue;
		goto(path);
	};

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

	const deviceStatus = writable<DeviceStatus>({
		usb_devices: [],
		sata_devices: [],
		cpu_usage: 0,
		ram_usage: 0
	});

	const loading = writable(true);
	let errorMessage: string | null = null;

	const getFormattedDateTime = () => {
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
	<AppRail class="w-[5.35rem]">
		<svelte:fragment slot="lead">
			<AppRailAnchor href="/">
				<img src="/rangers-logo.png" alt="Rangers Logo" class="logo fade-text" />
			</AppRailAnchor>
		</svelte:fragment>

		<AppRailTile
			on:click={() => handleNavigation('/dashboard', 0)}
			bind:group={currentTile}
			name="tile-1"
			value={0}
			title="Kopírování"
		>
			<svelte:fragment slot="lead">
				<div class="icons"><FaCopy /></div>
			</svelte:fragment>
			<span>Kopírování</span>
		</AppRailTile>

		<AppRailTile
			on:click={() => handleNavigation('/dashboard/disk_info', 1)}
			bind:group={currentTile}
			name="tile-2"
			value={1}
			title="Hashe"
		>
			<svelte:fragment slot="lead">
				<div class="icons"><FaHdd /></div>
			</svelte:fragment>
			<span>Disk Info</span>
		</AppRailTile>

		<AppRailTile
			on:click={() => handleNavigation('/dashboard', 2)}
			bind:group={currentTile}
			name="tile-3"
			value={2}
			title="Formátování"
		>
			<svelte:fragment slot="lead">
				<div class="icons"><FaBroom /></div>
			</svelte:fragment>
			<span>Formátování</span>
		</AppRailTile>

		<AppRailTile
			on:click={() => handleNavigation('/dashboard', 3)}
			bind:group={currentTile}
			name="tile-4"
			value={3}
			title="Historie"
		>
			<svelte:fragment slot="lead">
				<div class="icons"><FaHistory /></div>
			</svelte:fragment>
			<span>Historie</span>
		</AppRailTile>

		<AppRailTile
			on:click={() => handleNavigation('/dashboard/pre_configs', 4)}
			bind:group={currentTile}
			name="tile-5"
			value={4}
			title=""
		>
			<svelte:fragment slot="lead">
				<div class="icons"><FaSlidersH /></div>
			</svelte:fragment>
			<span>Konfigurace</span>
		</AppRailTile>

		<AppRailTile
			on:click={() => handleNavigation('/dashboard', 5)}
			bind:group={currentTile}
			name="tile-6"
			value={5}
			title="Nastavení"
		>
			<svelte:fragment slot="lead">
				<div class="icons"><FaCog /></div>
			</svelte:fragment>
			<span>Nastavení</span>
		</AppRailTile>
	</AppRail>

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
								<FaUsb />
							</div>
							<span class="device-name">{device.interface}</span>
						</span>
					{/each}

					{#each $deviceStatus.sata_devices as device}
						<span title={`SATA Device Interface: ${device.interface}`} class="device-indicator">
							<div class="connected-icon" style="width: 20px;">
								<FaHdd />
							</div>
							<span class="device-name">{device.interface}</span>
						</span>
					{/each}
				{/if}
			</div>

			<div class="header-right">
				<span class="sysinfo-span">
					<div class="connected-icon" style="width: 20px;">
						<FaCloudscale />
					</div>
					<span class="device-name">CPU: {$deviceStatus.cpu_usage.toFixed(1)}%</span>
					<span class="device-name">RAM: {$deviceStatus.ram_usage.toFixed(1)}%</span>
				</span>
			</div>
		</header>

		<main class="main-content">
			<slot />
		</main>
	</div>

	<button class="floating-div" on:click={() => modalStore.trigger(modal)}>
		<div style="width: 10px; margin-right: 5px;"><FaAngleDoubleUp /></div>
		Procesy
		<div style="width: 10px;margin-left: 5px;"><FaAngleDoubleUp /></div>
	</button>
</div>

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
		background-color: rgba(var(--color-surface-800) / 1);
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
		margin-left: 0.3rem;
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

	.icons {
		height: 2rem;
		width: 2rem;
		margin: auto;
		display: flex;
		align-items: center;
		justify-content: center;
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
