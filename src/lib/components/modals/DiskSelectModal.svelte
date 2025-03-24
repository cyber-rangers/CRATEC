<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { deviceStore } from '$lib/stores/deviceStore';
	import { copyRunStore } from '$lib/stores/copyRunStore';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Usb, HardDrive } from 'lucide-svelte';

	export let openState: boolean = false;
	export let meta;


	function modalClose() {
		openState = false;
	}

	onMount(() => {
		setTimeout(() => {
			const modalElement = document.querySelector('.table-wrap');
			if (modalElement) {
				modalElement.scrollTop = 0;
			}
		}, 10);
	});

	let allDisks = [];
	let index = 0;
	
	let selectedDisk = null;
	let deviceDetails = {};

	$: allDisks = [
		...$deviceStore.usb_devices.map((d) => ({ ...d, type: 'usb' })),
		...$deviceStore.sata_devices.map((d) => ({ ...d, type: 'sata' }))
	];

	async function fetchDeviceDetails(device) {
		const command = device.type === 'usb' ? 'get_usb_device_details' : 'get_hdd_details';
		try {
			deviceDetails[device.interface] = await invoke(command, {
				devicePath: device.interface
			});
		} catch (error) {
			console.error('Failed to fetch device details:', error);
		}
	}

	function nextDisk() {
		if (index < allDisks.length - 1) index++;
		else index = 0;
		fetchDeviceDetails(allDisks[index]);
	}
	function prevDisk() {
		if (index > 0) index--;
		else index = allDisks.length - 1;
		fetchDeviceDetails(allDisks[index]);
	}
	function selectDisk() {
		selectedDisk = allDisks[index];
	}
	function confirmSelection() {
		if (!selectedDisk) return;
		const side = meta?.side;
		if (side === 'input') {
			copyRunStore.update((state) => ({ ...state, inputDisk: selectedDisk }));
		} else if (side === 'output') {
			copyRunStore.update((state) => {
				const exists = state.outputDisks.some((d) => d.interface === selectedDisk.interface);
				if (!exists) {
					return {
						...state,
						outputDisks: [...state.outputDisks, selectedDisk]
					};
				}
				return state;
			});
		}
		openState = false;
	}

	onMount(() => {
		if (allDisks.length > 0) {
			fetchDeviceDetails(allDisks[0]);
		}
	});
</script>

<Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		{#if allDisks.length > 0}
			<div class="modal">
				<div class="modal-body">
					<button class="nav left" onclick={prevDisk}>&#9664;</button>
					<div
						class="carousel"
						onclick={selectDisk}
						class:selected={selectedDisk === allDisks[index]}
					>
						{#if allDisks[index].type === 'usb'}
							<div class="connected-icon" style="width: 20%;">
								<Usb />
							</div>
						{:else}
							<div class="connected-icon" style="width: 20%;">
								<HardDrive />
							</div>
						{/if}
						<span style="height: 20px;"></span>
						<span class="span-model"
							>{deviceDetails[allDisks[index].interface]?.device_model ||
								deviceDetails[allDisks[index].interface]?.id_model}</span
						>
						<span class="span-infoheader">Rozhraní:</span>
						<span class="span-info">{allDisks[index].interface}</span>
						<span class="span-infoheader">Seriové číslo:</span>
						<span class="span-info"
							>{deviceDetails[allDisks[index].interface]?.serial_number ||
								deviceDetails[allDisks[index].interface]?.id_serial_short}</span
						>
						<span class="span-infoheader">Vendor:</span>
						<span class="span-info"
							>{deviceDetails[allDisks[index].interface]?.user_capacity ||
								deviceDetails[allDisks[index].interface]?.id_vendor}</span
						>
						{#if selectedDisk === allDisks[index]}
							<button class="confirm" onclick={confirmSelection}>Potvrdit</button>
						{/if}
					</div>
					<button class="nav right" onclick={nextDisk}>&#9654;</button>
				</div>
			</div>
		{/if}
	{/snippet}
</Modal>

<style lang="postcss">
	.modal {
		background: transparent;
		padding: 1.5rem;
		border-radius: 8px;
		width: 450px;
		height: 400px;
		margin: auto;
		text-align: center;
	}
	.modal-body {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 1rem;
	}
	.carousel {
		width: 100%;
		height: 300px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		background: rgba(var(--color-surface-700) / 1);
		border-radius: 12px;
		border: 2px solid #ccc;
		cursor: pointer;
		transition: background 0.3s ease;
		padding: 1rem;
	}
	.carousel:hover {
		background: rgba(0, 0, 0, 0.1);
	}
	.carousel.selected {
		background: rgba(0, 0, 0, 0.1);
	}
	.icon {
		width: 40px;
		height: 40px;
		margin-bottom: 0.5rem;
	}
	.confirm {
		margin-top: 0.5rem;
		padding: 0.5rem 1rem;
		background: green;
		color: white;
		border: none;
		border-radius: 5px;
		cursor: pointer;
	}
	.confirm:hover {
		background: darkgreen;
	}
	.nav {
		background: transparent;
		border: none;
		font-size: 40px;
		cursor: pointer;
		color: rgba(var(--color-primary-500) / 1);
		outline: none;
	}
	.nav:focus,
	.nav:active {
		outline: none !important;
	}

	.span-model {
		font-weight: 900;
		font-size: 25px;
	}

	.span-infoheader {
		padding-top: 10px;
		font-size: 18px;
	}

	.span-info {
		font-weight: 700;
		font-size: 18px;
	}
</style>
