<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { Settings, Power, RefreshCw } from 'lucide-svelte';

	// Exportujeme stav modalu pro ovládání z rodičovské stránky
	export let openState: boolean = false;
</script>

<Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase=""
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet trigger()}
		<!-- Trigger bude ovládaný kliknutím na logo v navigaci -->
	{/snippet}
	{#snippet content()}
		<article class="flex h-[23vh] w-[40vw] items-center justify-around gap-4">
			<button
				type="button"
				class="btn preset-tonal flex flex-col items-center rounded-lg w-32 h-32"
				style="background-color: var(--color-surface-700)"
			>
				<Settings class="h-6 w-6" />
				<span class="mt-2 text-sm">Nastavení</span>
			</button>
			<button
				type="button"
				class="btn preset-tonal flex flex-col items-center rounded-lg w-32 h-32"
				style="background-color: var(--color-surface-700)"
			>
				<Power class="h-6 w-6" />
				<span class="mt-2 text-sm">Vypnutí</span>
			</button>
			<button
				type="button"
				class="btn preset-tonal flex flex-col items-center rounded-lg w-32 h-32"
				style="background-color: var(--color-surface-700)"
			>
				<RefreshCw class="h-6 w-6" />
				<span class="mt-2 text-sm">Restart</span>
			</button>
		</article>
	{/snippet}
</Modal>
