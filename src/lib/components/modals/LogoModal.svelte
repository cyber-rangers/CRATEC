<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { Settings, Power, RefreshCw } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
  
	// Exportujeme stav modalu pro ovládání z rodičovské stránky
	export let openState: boolean = false;
  
	async function shutdownSystem() {
	  try {
		await invoke('shutdown_system');
	  } catch (error) {
		console.error('Chyba při vypnutí systému:', error);
	  }
	}
  
	async function restartSystem() {
	  try {
		await invoke('restart_system');
	  } catch (error) {
		console.error('Chyba při restartu systému:', error);
	  }
	}
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
		  on:click={shutdownSystem}
		>
		  <Power class="h-6 w-6" />
		  <span class="mt-2 text-sm">Vypnutí</span>
		</button>
		<button
		  type="button"
		  class="btn preset-tonal flex flex-col items-center rounded-lg w-32 h-32"
		  style="background-color: var(--color-surface-700)"
		  on:click={restartSystem}
		>
		  <RefreshCw class="h-6 w-6" />
		  <span class="mt-2 text-sm">Restart</span>
		</button>
	  </article>
	{/snippet}
  </Modal>
  