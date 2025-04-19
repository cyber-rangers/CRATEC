<script lang="ts">
	import { Modal, Progress } from '@skeletonlabs/skeleton-svelte';
	import { SquareX, Send, Lock, LockOpen, ChevronUp } from 'lucide-svelte';
	import { Resource, invoke } from '@tauri-apps/api/core';
	import { Toaster, createToaster } from '@skeletonlabs/skeleton-svelte';
	import { runningProcessesStore } from '$lib/stores/processStore';

	export let isOpen: boolean = false;

	let input: string = '';
	let firstInput: string | null = null;
	let locked: boolean = false;

	const toaster = createToaster({
		placement: 'top'
	});

	function closeDrawer() {
		isOpen = false;
		reset();
	}

	function reset() {
		input = '';
		firstInput = null;
	}

	function openDrawer() {
		isOpen = true;
	}

	function handleDigit(d: string) {
		if (input.length < 6) input += d;
	}

	function handleClear() {
		input = '';
	}

	async function handleSubmit() {
		if (!locked) {
			if (firstInput === null) {
				if (input.length < 4) {
					toaster.error({ title: 'Zadejte minimálně 4 číslice!' });
					return;
				}
				firstInput = input;
				input = '';
				toaster.info({ title: 'Potvrďte kód!' });
				return;
			}
			if (input !== firstInput) {
				toaster.error({ title: 'Kódy se neshodují!' });
				firstInput = null;
				input = '';
				return;
			}
			await invoke('lock_system', { code: input });
			locked = true;
			reset();
			toaster.success({ title: 'Systém byl úspěšně zamčen!' });
		} else {
			const ok: boolean = await invoke('unlock_system', { code: input });
			if (ok) {
				locked = false;
				isOpen = false;
				reset();
				toaster.success({ title: 'Systém odemčen!' });
			} else {
				toaster.error({ title: 'Neplatný kód!' });
				input = '';
			}
		}
	}

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
</script>

<Modal
	open={isOpen}
	onOpenChange={(e) => (isOpen = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="preset-gradient-one h-screen w-screen p-4 flex flex-col"
	positionerJustify="justify-start"
	positionerAlign="items-start"
	positionerPadding=""
	transitionsPositionerIn={{ y: -1000, duration: 200 }}
	transitionsPositionerOut={{ y: -1000, duration: 200 }}
>
	{#snippet trigger()}
		Open Fullscreen Drawer
	{/snippet}

	{#snippet content()}
		<div class="relative flex h-full w-full flex-row">
			<!-- Levá polovina (vstup, logo a numpad) -->
			<div class="relative flex w-1/2 flex-col items-center justify-center">
				<!-- Kratší vertikální čára -->
				<div class="bg-tertiary-500 absolute top-1/12 right-0 z-10 h-6/7 w-0.5"></div>
				<!-- Logo zámku nad vstupem - změněna barva na primary-500 -->
				<div class="mb-8 flex items-center justify-center">
					{#if locked}
						<Lock class="text-primary-500 h-12 w-12" />
					{:else}
						<LockOpen class="text-primary-500 h-12 w-12" />
					{/if}
				</div>
				<!-- Vstup -->
				<div class="mb-6 flex flex-col items-center">
					<input
						type="password"
						class="input preset-outlined-primary-500 h-12 w-55 rounded px-3 py-2 text-lg"
						bind:value={input}
						readonly
						placeholder={locked
							? 'Zadejte kód odemčení'
							: firstInput
								? 'Potvrďte kód'
								: 'Zadejte kód'}
					/>
				</div>
				<!-- Numpad -->
				<div class="grid grid-cols-3 gap-4">
					{#each ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as num}
						<button
							class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
							on:click={() => handleDigit(num)}
						>
							{num}
						</button>
					{/each}
					<button
						class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
						on:click={handleClear}
					>
						<SquareX />
					</button>
					<button
						class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
						on:click={() => handleDigit('0')}
					>
						0
					</button>
					<button
						class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
						on:click={handleSubmit}
					>
						<Send />
					</button>
				</div>
				<!-- Půlkruh a zavírací tlačítko dole -->
				{#if !locked}
					<div class="close-half-circle-fixed z-30">
						<button class="close-btn" on:click={closeDrawer} title="Zavřít">
							<ChevronUp class="text-primary-500 h-8 w-8" />
						</button>
					</div>
				{/if}
			</div>

			<!-- Pravá polovina (aktivní procesy) -->
			<div class="flex w-1/2 flex-col items-center justify-start overflow-y-auto px-4 pt-8">
				{#each $runningProcessesStore as process (process.id)}
					<div class="mb-4 flex w-[90%] flex-col rounded-lg bg-white shadow-md">
						<!-- Top section: stav, rychlost -->
						<div class="top-section bg-surface-600 flex w-full rounded-t-lg px-4 py-2">
							<div class="w-1/2 text-left">
								<span class="font-semibold">Status:</span>
								{process.status}
							</div>
							<div class="w-1/2 text-right">
								<span class="font-semibold">
									{#if process.progress_perc === 100 && process.status === 'done'}
										N/A
									{:else}
										{process.speed.toFixed(1)} MiB/s
									{/if}
								</span>
							</div>
						</div>
						<!-- Middle section: vstupní a výstupní disky -->
						<div
							class="middle-section bg-surface-800 flex w-full items-center justify-between px-4 py-2"
						>
							<div class="flex items-center gap-2">
								<span class="bg-surface-800 rounded px-2 py-1 text-xs">IN</span>
								<span class="text-xs">{process.source_disk.name}</span>
							</div>
							<div class="bg-surface-800 flex w-1/3 items-center justify-center rounded p-1">
								<div class="arrow">
									<span></span>
									<span></span>
									<span></span>
								</div>
							</div>
							<div class="flex flex-col gap-1">
								{#each process.destination_disks as disk}
									<div class="flex items-center gap-2">
										<span class="bg-surface-800 rounded px-2 py-1 text-xs">OUT</span>
										<span class="text-xs">{disk.interface}</span>
									</div>
								{/each}
							</div>
						</div>
						<!-- Bottom section: progress bar a čas -->
						<div
							class="bottom-section bg-surface-600 flex w-full items-center rounded-b-lg px-4 py-4"
						>
							<div class="flex-1">
								<Progress value={process.progress_perc} max={100} meterBg="bg-primary-500">
									{process.progress_perc}%
								</Progress>
							</div>
							<p class="ml-4 text-xs">
								{#if process.progress_perc === 100 && process.status === 'done'}
									N/A
								{:else}
									{formatTime(process.progress_time)}
								{/if}
							</p>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/snippet}
</Modal>

<Toaster {toaster}></Toaster>

<style>
	:global(.preset-gradient-one) {
		background-image: linear-gradient(45deg, var(--color-surface-900), var(--color-surface-600));
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
	/* Půlkruh dole - fixed pozice */
	.close-half-circle-fixed {
		position: fixed;
		left: 50%;
		bottom: 0;
		transform: translateX(-50%);
		width: 80px;
		height: 40px;
		background: var(--color-surface-600);
		border-top-left-radius: 80px;
		border-top-right-radius: 80px;
		border-bottom-left-radius: 0;
		border-bottom-right-radius: 0;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.08);
		z-index: 100;
	}
	.close-btn {
		background: none;
		outline: none;
		cursor: pointer;
		margin-top: -18px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.2s;
	}
	.close-btn:hover {
		background: var(--color-primary-100);
		border-radius: 50%;
	}
</style>
