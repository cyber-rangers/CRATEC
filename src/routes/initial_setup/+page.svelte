<script lang="ts">
	import { t, locale, getAvailableLanguages } from '$lib/translation/i18n';
	import { settings } from '$lib/stores/settingsStore';
	import { goto } from '$app/navigation';
	import { fade } from 'svelte/transition';
	import translations from '$lib/translation/translations';
	import { onMount } from 'svelte';

	let languages: string[] = getAvailableLanguages();
	let languageChoices: string[] = [];
	let currentIndex = 0;
	let currentLanguageChoice: string = '';
	let isFadingOut = false;

	$: {
		languageChoices = languages.map((lang) => (translations as any)[lang]['language-choice']);
	}

	const changeLanguageChoice = () => {
		setInterval(() => {
			isFadingOut = true;

			setTimeout(() => {
				currentIndex = (currentIndex + 1) % languageChoices.length;
				currentLanguageChoice = languageChoices[currentIndex];
				isFadingOut = false;
			}, 400);
		}, 3000);
	};

	onMount(() => {
		currentLanguageChoice = languageChoices[currentIndex];
		changeLanguageChoice();
	});

	function selectLanguage(lang: string) {
		settings.update((prev) => ({ ...prev, language: lang }));
		locale.set(lang);
		goto('/initial_setup/report_language');
	}
</script>

<div class="h-screen flex flex-col items-center justify-center relative">
	{#if !isFadingOut}
		<h2 transition:fade={{ duration: 400 }}>
			<span
				class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone"
				>{currentLanguageChoice}</span
			>
		</h2>
	{/if}

	<figure class="flex space-x-4 justify-center mt-6">
		{#each languages as lang}
			<button
				type="button"
				class="btn variant-ghost-primary w-48 h-16 font-semibold text-lg"
				on:click={() => selectLanguage(lang)}
			>
				{#if lang === 'cs'}
					Čeština
				{/if}
				{#if lang === 'en'}
					English
				{/if}
				{#if lang === 'de'}
					Deutsch
				{/if}
			</button>
		{/each}
	</figure>
</div>

<style lang="postcss">
	h2 {
		font-size: 3rem;
		font-weight: 1000;
		margin-bottom: 5rem;
	}
	.h-screen {
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
	}
</style>
