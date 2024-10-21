<script lang="ts">
	import { t, locale, getAvailableLanguages } from '$lib/translation/i18n';
	import { settings } from '$lib/stores/settingsStore';
	import { goto } from '$app/navigation';

	let languages: string[] = getAvailableLanguages(); 

	function selectLanguage(lang: string) {
	settings.update((prev) => ({ ...prev, reportLanguage: lang }));
	  goto('/initial_setup/image_format');
	}
</script>

<div class="h-screen flex flex-col items-center justify-center">
	<div class="text-center w-full px-4"> 
	  <h2><span
		class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone"
		>{$t('report-language-choice')}</span></h2>

	  <figure class="flex space-x-4 justify-center mt-6">
		{#each languages as lang}
		  <button
			type="button"
			class="btn variant-ghost-primary w-32 h-16 font-semibold"
			on:click={() => selectLanguage(lang)}
		  >
			{#if lang === 'cs'} Čeština {/if}
			{#if lang === 'en'} English {/if}
			{#if lang === 'de'} Deutsch {/if}
		  </button>
		{/each}
	  </figure>
	</div>
</div>

<style lang="postcss">
    h2 {
        font-size: 3rem;
        line-height: 1.2;
		font-weight: 1000;
        margin-bottom: 5rem;
        padding-top: 0.5rem; /* Add padding to prevent clipping */
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
