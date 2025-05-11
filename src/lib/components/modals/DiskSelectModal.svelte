<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Usb, HardDrive, ArrowLeft, ArrowRight } from 'lucide-svelte';
	import { deviceStore, type DeviceBase } from '$lib/stores/deviceStore';
	import { copyRunStore } from '$lib/stores/copyRunStore';
  
	// Očekáváme, že se modal volá s prop "side" ("input" nebo "output").
	// Výchozí hodnota nastavíme na "output".
	export let side: 'input' | 'output' = 'output';
	export let openState: boolean = false;
  

	// Disková struktura (DeviceBase + pole type)
	type Disk = DeviceBase & { type: 'usb' | 'sata' };
  
	console.log('side:', side);
  
	// Reaktivně získáváme disky z deviceStore podle side
	$: allDisks = [
	  ...$deviceStore.usb_devices.filter((d) => d.side === side).map((d): Disk => ({ ...d, type: 'usb' })),
	  ...$deviceStore.sata_devices.filter((d) => d.side === side).map((d): Disk => ({ ...d, type: 'sata' }))
	];

	// Pro výstupní disky filtrujeme již zvolené, pro vstup necháváme všechny
	$: availableDisks = side === 'output'
		? allDisks.filter((disk) => !$copyRunStore.outputDisks.some((selected) => selected.interface === disk.interface))
		: allDisks;

	console.log('allDisks:', allDisks);
	$: console.log('deviceStore:', $deviceStore);
  
	let carouselContainer: HTMLDivElement;
	let currentIndex: number = 0;
	let selectedDisk: Disk | null = null;
	let deviceDetails: Record<string, any> = {};
  
	async function fetchDeviceDetails(device: Disk) {
	  const command = device.type === 'usb' ? 'get_usb_device_details' : 'get_hdd_details';
	  try {
		const details = await invoke(command, { devicePath: device.interface });
		deviceDetails[device.interface] = details;
		console.log(`Details for ${device.interface}:`, details);
	  } catch (error) {
		console.error('Failed to fetch device details:', error);
	  }
	}
  
	// Posune o jeden disk (250px)
	function carouselLeft() {
	  if (!carouselContainer || availableDisks.length === 0) return;
	  carouselContainer.scrollLeft -= 250;
	  currentIndex = Math.floor(carouselContainer.scrollLeft / 250);
	  console.log('Carousel left, currentIndex:', currentIndex);
	}
  
	function carouselRight() {
	  if (!carouselContainer || availableDisks.length === 0) return;
	  carouselContainer.scrollLeft += 250;
	  currentIndex = Math.floor(carouselContainer.scrollLeft / 250);
	  console.log('Carousel right, currentIndex:', currentIndex);
	}
  
	function carouselThumbnail(index: number) {
	  if (!carouselContainer) return;
	  carouselContainer.scrollLeft = carouselContainer.clientWidth * index;
	  currentIndex = index;
	  console.log('Carousel thumbnail clicked, currentIndex:', currentIndex);
	}
  
	function confirmSelection(disk: Disk) {
	  if (!disk) return;
	  console.log('Confirming selection for disk:', disk);
	  if (side === 'input') {
		copyRunStore.update((state) => ({ ...state, inputDisk: disk }));
	  } else if (side === 'output') {
		copyRunStore.update((state) => {
		  // Přidáme disk jenom pokud není již přidán a pokud je méně než 2 disky zvoleno
		  if (state.outputDisks.length < 2 && !state.outputDisks.some((d) => d.interface === disk.interface)) {
			return { ...state, outputDisks: [...state.outputDisks, disk] };
		  }
		  return state;
		});
	  }
	  openState = false;
	}
  
	// Převod velikosti disku: počítáme počet bajtů = sector_count * sector_size a převedeme na GB
	function formatSizeGB(sector_count: number | null, sector_size: number | null): string {
	  if (!sector_count || !sector_size) return '0 GB';
	  const bytes = sector_count * sector_size;
	  return (bytes / (1024 ** 3)).toFixed(2) + ' GB';
	}
  </script>
  
  <Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-transparent p-4 space-y-4 max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
  >
	{#snippet content()}
	  <div class="w-full">
		<!-- Carousel -->
		<div class="card grid grid-cols-[auto_1fr_auto] items-center gap-4 p-4">
		  <!-- Button: Left -->
		  <button
			type="button"
			class="btn rounded-full w-12 h-12 preset-filled-primary-500 preset-filled"
			on:click={carouselLeft}
			aria-label="Předchozí disk"
			disabled={availableDisks.length < 2}
		  >
			<ArrowLeft size={16} />
		  </button>
		  <!-- Carousel Container -->
		  <div
			bind:this={carouselContainer}
			data-carousel
			class="flex snap-x snap-mandatory overflow-x-auto scroll-smooth w-[250px]"
		  >
			{#each availableDisks as disk, i (disk.interface)}
			  <div
				class="rounded-container flex w-[250px] h-[200px] bg-surface-900 cursor-pointer snap-center flex-col items-center justify-center border-2 p-2"
				on:click={() => {
				  console.log('Selected disk:', disk);
				  confirmSelection(disk);
				}}
			  >
				{#if disk.type === 'usb'}
				  <div class="connected-icon" style="width: 40px;">
					<Usb />
				  </div>
				{:else}
				  <div class="connected-icon" style="width: 40px;">
					<HardDrive />
				  </div>
				{/if}
				<p class="disk-name">{disk.name || 'Neznámý disk'}</p>
				<p class="disk-serial">
				  Sériové číslo:<br />
				  {disk.serial || 'N/A'}
				</p>
				<p class="disk-size">
				  Velikost:<br />
				  {formatSizeGB(disk.sector_count, disk.sector_size)}
				</p>
			  </div>
			{/each}
		  </div>
		  <!-- Button: Right -->
		  <button
			type="button"
			class="btn rounded-full w-12 h-12 preset-filled-primary-500"
			on:click={carouselRight}
			aria-label="Další disk"
			disabled={availableDisks.length < 2}
		  >
			<ArrowRight size={16} />
		  </button>
		</div>
	  </div>
	{/snippet}
  </Modal>
  
  <style lang="postcss">
	[data-carousel] {
	  scroll-snap-type: x mandatory;
	  display: flex;
	  overflow-x: auto;
	  scroll-behavior: smooth;
	  -ms-overflow-style: none; /* IE and Edge */
	  scrollbar-width: none; /* Firefox */
	}
	[data-carousel]::-webkit-scrollbar {
	  display: none; /* Chrome, Safari, Opera */
	}
	.snap-center {
	  scroll-snap-align: center;
	  flex-shrink: 0;
	}
	.confirm-container {
	  margin-top: 1rem;
	  text-align: center;
	}
	.confirm {
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
	.no-disks-info {
	  color: var(--color-primary-500);
	  font-weight: bold;
	  text-align: center;
	  padding: 2rem;
	}
	.disk-name {
	  font-weight: bold;
	  font-size: 1.25rem;
	  margin-top: 0.5rem;
	}
	.disk-serial,
	.disk-size,
	.thumb-name {
	  font-size: 0.9rem;
	}
	.disk-serial {
	  font-size: 0.9rem;
	  text-align: center;
	}
	.modal {
	  background: transparent;
	  padding: 1.5rem;
	  border-radius: 8px;
	  width: 450px;
	  margin: auto;
	  text-align: center;
	}
	.modal-body {
	  display: flex;
	  align-items: center;
	  justify-content: center;
	  gap: 1rem;
	}
	.btn-icon {
	  border: none;
	  cursor: pointer;
	}
	.thumbnail {
	  display: flex;
	  flex-direction: column;
	  align-items: center;
	}
	.thumb-name {
	  margin-top: 0.25rem;
	}
  </style>
