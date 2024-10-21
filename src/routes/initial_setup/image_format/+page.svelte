<script lang="ts">
	import { t, locale, getAvailableLanguages } from '$lib/translation/i18n';
	import { settings } from '$lib/stores/settingsStore';
	import { goto } from '$app/navigation';

	function selectImageFormat(format: string) {
    settings.update((prev) => ({ ...prev, imageFormat: format}));
	  goto('/initial_setup/hash_algo');
	}
</script>

<div class="h-screen flex flex-col items-center justify-center">
	<div class="text-center w-full px-4"> 
	  <h2><span
		class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone"
		>{$t('image-format')}</span></h2>

	  <figure class="flex space-x-4 justify-center mt-6">
		<button
			type="button"
			class="btn variant-ghost-primary w-32 h-16 font-semibold"
			on:click={() =>  selectImageFormat("DD")}>
			DD
		  </button>
          <button
			type="button"
			class="btn variant-ghost-primary w-32 h-16 font-semibold"
			on:click={() => selectImageFormat("E01")}>
			E01
		  </button>
          <button
			type="button"
			class="btn variant-ghost-primary w-32 h-16 font-semibold"
			on:click={() => selectImageFormat("Ex01")}>
			Ex01
		  </button>
	  </figure>
	</div>
</div>

<style lang="postcss">
    h2 {
        font-size: 3rem;
		font-weight: 1000;
        line-height: 1.2; 
        margin-bottom: 5rem;
        padding-top: 0.5rem; 
    }

	.h-screen {
		display: flex;
		align-items: center;
		justify-content: center;
	}

	button {
        @apply w-48 h-16 font-bold bg-opacity-50 text-lg;
    }

	.text-center {
		max-width: 100%; 
		overflow: hidden; 
	}
</style>
