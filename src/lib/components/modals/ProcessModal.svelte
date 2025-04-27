<script lang="ts">
	import { Modal, Progress } from '@skeletonlabs/skeleton-svelte';
	import {
		LoaderCircle,
		CircleX,
		SquareCode,
		Usb,
		HardDrive,
		CircleAlert,
		CircleCheck,
		X
	} from 'lucide-svelte';
	import { runningProcessesStore } from '$lib/stores/processStore';
	import { get } from 'svelte/store';
	import { deviceStore } from '$lib/stores/deviceStore';

	// Stav modalu pro seznam procesů a drawer s logy
	export let openState: boolean = false;
	let drawerState: boolean = false;

	// Vybraný proces (uložené id) – po kliknutí se uloží id procesu, pro který se má drawer zobrazit log
	let selectedProcessId: string | null = null;

	// Reaktivně vyhledáme vybraný proces ze store
	$: selectedProcess = $runningProcessesStore.find((p) => p.id === selectedProcessId);

	$: console.log('selectedProcess change:', selectedProcess);

	function modalClose() {
		openState = false;
	}

	function drawerClose() {
		drawerState = false;
	}

	// Otevře drawer pro daný proces (volá se z tlačítka v seznamu procesů)
	function openDrawerForProcess(id: string) {
		selectedProcessId = id;
		drawerState = true;
	}

	// Pomocná funkce pro formátování času (v sekundách) do čitelného formátu (např. "1h 16m 5s")
	function formatTime(totalSeconds: number): string {
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		const seconds = totalSeconds % 60;
		let result = '';
		if (hours > 0) {
			result += hours + 'h ';
		}
		if (minutes > 0 || hours > 0) {
			result += minutes + 'm ';
		}
		result += seconds + 's';
		return result;
	}

	function getDeviceNameByInterface(iface: string): string {
		const devices = [...get(deviceStore).usb_devices, ...get(deviceStore).sata_devices];
		const found = devices.find((d) => d.interface === iface);
		return found?.name || iface;
	}

	function getDeviceNameByMountpoint(mount: string): string {
		const devices = [...get(deviceStore).usb_devices, ...get(deviceStore).sata_devices];
		const found = devices.find((d) => d.mountpoint === mount);
		return found?.name || mount;
	}

	function getDeviceNameByInterfacePath(path: string): string {
		// Získá poslední část cesty
		const iface = path.split('/').pop();
		const devices = [...get(deviceStore).usb_devices, ...get(deviceStore).sata_devices];
		const found = devices.find((d) => d.interface === iface);
		return found?.name || iface || path;
	}
</script>

<!-- Modal pro zobrazení aktivních procesů -->
<Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl my-modal-content"
	contentClasses="w-full max-w-[500px]"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		<!-- Přidáno plovoucí zavírací tlačítko vpravo nahoře -->
		<button
			type="button"
			class="btn preset-filled-surface-500 absolute top-6 right-6 z-50 flex h-10 w-10 items-center justify-center rounded-full p-0"
			on:click={modalClose}
			title="Zavřít"
		>
			<X size={20} />
		</button>
		{#if $runningProcessesStore.length === 0}
			<p class="text-center">Žádné aktivní procesy.</p>
		{:else}
			{#each $runningProcessesStore as process (process.id)}
				{@html (() => {
					console.log(process);
					return '';
				})()}
				<div class="process-box mb-4 flex flex-col rounded-lg bg-white shadow-md">
					<!-- Top section: stav, rychlost a tlačítko pro otevření draweru -->
					<div class="top-section bg-surface-600 flex w-full rounded-t-lg py-4">
						<div class="w-1/3 text-center">
							<div class="flex items-center justify-center gap-2">
								<p class="flex items-center gap-1">
									Status: {process.status}
									{#if process.status === 'running'}
										<LoaderCircle class="animate-spin" />
									{:else if process.status === 'error'}
										<CircleAlert />
									{:else if process.status === 'done'}
										<CircleCheck />
									{/if}
								</p>
							</div>
						</div>
						<div class="w-1/3 text-center">
							<p>
								{#if process.status === 'error'}
									N/A
								{:else if process.progress_perc === 100 && process.status === 'done'}
									N/A
								{:else}
									{process.speed.toFixed(1)} MiB/s
								{/if}
							</p>
						</div>
						<div class="flex w-1/3 items-center justify-end gap-2 pr-4 text-right">
							<button on:click={() => openDrawerForProcess(process.id)}><SquareCode /></button>
						</div>
					</div>

					<!-- Middle section: vstupní a výstupní disky -->
					<div class="middle-section flex w-full items-stretch py-1 text-center">
						<!-- Source disk (levý sloupec) -->
						<div
							class="bg-surface-700-900 flex w-1/3 flex-col items-center justify-center gap-2 rounded p-1"
						>
							<div class="flex items-center gap-2">
								<div class="connected-icon"></div>
								{#if getDeviceNameByInterfacePath(typeof process.source_disk === 'string' ? process.source_disk : process.source_disk?.interface || '')
									?.toLowerCase()
									.includes('usb')}
									<Usb size={25} />
								{:else}
									<HardDrive size={25} />
								{/if}
							</div>
							<p class="text-sm">
								{getDeviceNameByInterfacePath(
									typeof process.source_disk === 'string'
										? process.source_disk
										: process.source_disk?.interface || ''
								)}
							</p>
						</div>

						<!-- Arrow (prostřední sloupec) -->
						<div
							class="bg-surface-700-900 flex w-1/3 flex-col items-center justify-center rounded p-1"
						>
							<div class="arrow">
								<span></span>
								<span></span>
								<span></span>
							</div>
						</div>

						<!-- Destination disks (pravý sloupec) -->
						<div
							class="bg-surface-700-900 flex w-1/3 flex-col items-center justify-center gap-1 rounded p-1"
						>
							{#each process.destination_disks as disk}
								<div class="flex items-center gap-2">
									<div class="connected-icon">
										{#if getDeviceNameByMountpoint(typeof disk === 'string' ? disk : disk?.mountpoint || '')
											?.toLowerCase()
											.includes('usb')}
											<Usb size={25} />
										{:else}
											<HardDrive size={25} />
										{/if}
									</div>
									<p class="text-sm">
										{getDeviceNameByMountpoint(
											typeof disk === 'string' ? disk : disk?.mountpoint || ''
										)}
									</p>
								</div>
							{/each}
						</div>
					</div>
					<!-- Bottom section: progress bar a čas -->
					<div
						class="bottom-section bg-surface-600 flex w-full items-center rounded-b-lg px-4 py-4"
					>
						<div class="flex-1">
							<Progress
								value={process.status === 'error' ? 0 : process.progress_perc}
								max={100}
								meterBg="bg-primary-500"
							>
								{process.status === 'error' ? '0%' : `${process.progress_perc}%`}
							</Progress>
						</div>
						<p class="ml-4 text-xs">
							{#if process.status === 'error' || (process.progress_perc === 100 && process.status === 'done')}
								N/A
							{:else}
								{formatTime(process.progress_time)}
							{/if}
						</p>
					</div>
				</div>
			{/each}
		{/if}
	{/snippet}
</Modal>

<!-- Drawer container with relative positioning -->
<div class="drawer-container">
	<!-- Drawer Modal pro zobrazení logu vybraného procesu -->
	<Modal
		open={drawerState}
		onOpenChange={(e) => (drawerState = e.open)}
		contentBase="bg-surface-100-900 p-4 space-y-4 shadow-xl w-[800px] h-screen relative"
		positionerJustify="justify-start"
		positionerAlign=""
		positionerPadding=""
		transitionsPositionerIn={{ x: -750, duration: 200 }}
		transitionsPositionerOut={{ x: -750, duration: 200 }}
	>
		{#snippet content()}
			<!-- Close button positioned within the modal content but at the right edge -->
			<button
				type="button"
				class="btn variant-filled absolute top-3 -right-12 flex h-9 w-9 items-center justify-center rounded-full p-0"
				on:click={drawerClose}
			>
				<X size={20} />
			</button>

			{#each $runningProcessesStore.filter((p) => p.id === selectedProcessId) as selected}
				<article>
					<pre class="pre max-h-142 overflow-y-auto">{#each selected.out_log as line}{line.trim()}
						{/each}</pre>
				</article>
			{:else}
				<p class="text-center">Žádný proces není vybrán.</p>
			{/each}
		{/snippet}
	</Modal>
</div>

<style lang="postcss">
	.process-box {
		background-color: var(--color-surface-800);
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	.animate-spin {
		animation: spin 2s linear infinite;
	}

	.arrow {
		position: relative;
		transform: rotate(270deg);
	}
	.arrow span {
		display: block;
		width: 3vw;
		height: 3vw;
		border-bottom: 15px solid var(--color-primary-500);
		border-right: 15px solid var(--color-primary-500);
		transform: rotate(45deg);
		margin: -10px;
		animation: animate 2s infinite;
	}
	.arrow span:nth-child(2) {
		animation-delay: -0.2s;
	}
	.arrow span:nth-child(3) {
		animation-delay: -0.4s;
	}
	@keyframes animate {
		0% {
			opacity: 0;
			transform: rotate(45deg) translate(-20px, -20px);
		}
		50% {
			opacity: 1;
		}
		100% {
			opacity: 0;
			transform: rotate(45deg) translate(20px, 20px);
		}
	}

	/* Drawer container positioning */
	.drawer-container {
		position: relative;
	}

	/* Close drawer button positioning */
	.close-drawer-btn {
		position: fixed;
		left: 810px;
		top: 10px;
		z-index: 50;
		padding: 8px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
	}
</style>
