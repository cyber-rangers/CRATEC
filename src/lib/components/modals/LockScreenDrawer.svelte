<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { SquareX, Send } from 'lucide-svelte';
	export let isOpen: boolean = false;

	function closeDrawer() {
		isOpen = false;
	}

	function openDrawer() {
		isOpen = true;
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
		<div class="flex h-full w-full flex-row">
			<!-- Levá polovina (vstup a numpad) -->
			<div class="border-primary-800 flex w-1/2 flex-col items-center justify-center border-r-2">
				<!-- Vstup nahoře -->
				<div class="mb-6">
					<input
						type="password"
						class="input border-primary-500 h-12 w-55 rounded border-2 px-3 py-2 text-lg"
						placeholder="Zadejte Heslo"
					/>
				</div>
				<!-- Numpad -->
				<div class="grid grid-cols-3 gap-4">
					{#each ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as num}
						<button class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white"
							>{num}</button
						>
					{/each}
					<button class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white"
						><SquareX /></button
					>
					<button class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white">0</button>
					<button class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white"
						><Send /></button
					>
				</div>
			</div>

			<!-- Pravá polovina (zatím prázdná) -->
			<div class="flex w-1/2 items-center justify-center">
				<button class="btn preset-filled mt-4" on:click={closeDrawer}>Zavřít</button>
			</div>
		</div>
	{/snippet}
</Modal>

<style>
	:global(.preset-gradient-one) {
		background-image: linear-gradient(
			45deg,
			var(--color-surface-900),
			var(--color-primary-900)
		);
		/* Pokud potřebujete přepsat transparent, přidejte: */
		background-color: unset;
	}
</style>
