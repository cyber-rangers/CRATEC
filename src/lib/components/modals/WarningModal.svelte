<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { TriangleAlert } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';

	export let openState: boolean = false;
	function modalClose() {
		openState = false;
	}
	function onContinue() {
		// zde případně akce pro pokračování
		openState = false;
	}
</script>

<Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase=""
	contentBase="card bg-yellow-400  p-4 pt-12 w-[60vw] space-y-6 shadow-xl relative"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		<div class="flex flex-col items-center">
			<TriangleAlert size={160} color="black" class="mb-4" />
			<h1 class="mb-2 text-center text-3xl font-bold text-black">HPA/DCO byl detekován!</h1>
			<p class="mb-8 max-w-md text-center text-base text-black">
				Byla detekována přítomnost HPA (Host Protected Area) a/nebo DCO (Device Configuration
				Overlay). HPA je oblast disku skrytá před operačním systémem, DCO umožňuje výrobcům měnit
				viditelnou kapacitu disku. Pokud budete pokračovat v akvizici, tyto oblasti nebudou zahrnuty
				do výsledných dat.
			</p>
		</div>
		<div class="mt-8 flex justify-between">
			<button
				class="rounded bg-black px-6 py-2 font-semibold text-yellow-400"
				on:click={modalClose}
			>
				Zrušit
			</button>
			<button
				class="rounded bg-black px-6 py-2 font-semibold text-yellow-400"
				on:click={onContinue}
			>
				Pokračovat
			</button>
		</div>
	{/snippet}
</Modal>
