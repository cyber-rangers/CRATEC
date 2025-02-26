<script lang="ts">
	import { goto } from '$app/navigation';
	import { FaArrowLeft, FaCheck } from 'svelte-icons/fa';
	import { getToastStore } from '@skeletonlabs/skeleton';
	import type { ToastSettings } from '@skeletonlabs/skeleton';
	import { t } from '$lib/translation/i18n';
	import { get } from 'svelte/store';
	import { settings } from '$lib/stores/settingsStore';

	const toastStore = getToastStore();

	let password = '';
	let confirmedPassword = '';
	let isSecondAttempt = false;
	let title = $t('set-passcode');

	function handleClick(number: string) {
		if (password.length < 4) {
			password += number;
		}
	}

	function deleteLast() {
		password = password.slice(0, -1);
	}

	function submitPassword() {
		if (password.length === 4) {
			if (!isSecondAttempt) {
				isSecondAttempt = true;
				title = get(t)('confirm-passcode');
				confirmedPassword = password;
				password = '';
			} else {
				if (confirmedPassword !== password) {
					const toast: ToastSettings = {
						message: get(t)('passcode-dont-match'),
						timeout: 5000,
						hideDismiss: true,
						background: 'variant-ghost-error'
					};
					toastStore.trigger(toast);
					isSecondAttempt = false;
					title = get(t)('set-passcode');
					password = '';
					confirmedPassword = '';
				} else {
					settings.update((prev) => ({ ...prev, passcode: password }));
					goto('/initial_setup/integrity_file');
				}
			}
		}
	}
</script>

<div class="inset-0 flex items-center justify-center">
	<div class="text-center w-full px-4">
		<h2><span class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone">{title}</span></h2>

		<div class="flex justify-center space-x-4 mb-6">
			{#each Array(4) as _, i}
				<div class="w-12 h-12 border-[2px] flex items-center justify-center text-3xl">
					{password[i] ? '*' : ''}
				</div>
			{/each}
		</div>

		<div class="justify-center grid grid-cols-3 gap-2 mt-6 place-items-center w-64 mx-auto">
			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('1')}>1</button>
			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('2')}>2</button>
			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('3')}>3</button>

			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('4')}>4</button>
			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('5')}>5</button>
			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('6')}>6</button>

			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('7')}>7</button>
			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('8')}>8</button>
			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('9')}>9</button>

			<button type="button" style="width: 2rem; color: rgba(var(--color-tertiary-500) / 1); opacity: 0.8;" on:click={deleteLast} aria-label="Delete last">
				<FaArrowLeft />
			</button>

			<button type="button" class="btn variant-ghost-primary" on:click={() => handleClick('0')}>0</button>

			<button style="width: 2rem; color: rgba(var(--color-tertiary-500) / 1);" on:click={submitPassword}>
				<FaCheck />
			</button>
		</div>
	</div>
</div>

<style lang="postcss">
	h2 {
		font-size: 2rem;
		margin-bottom: 2rem;
		margin-top: 0.5rem;
	}

	.flex {
		align-items: center;
		justify-content: center;
	}

	.grid {
		grid-template-columns: repeat(3, 1fr);
		gap: 0.5rem;
	}

	button {
		@apply w-16 h-16 font-bold text-xl;
	}

	.flex div {
		border-radius: 8px;
		border-color: rgba(var(--color-tertiary-500) / 1);
		font-weight: bold;
	}
</style>
