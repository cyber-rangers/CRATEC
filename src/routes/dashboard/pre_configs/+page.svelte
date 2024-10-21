<script lang="ts">
	import { FaPlus } from 'svelte-icons/fa';
	import { deviceStore } from '$lib/stores/deviceStore';
	import { get, writable } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/tauri';

	import { popup } from '@skeletonlabs/skeleton';
	import type { PopupSettings } from '@skeletonlabs/skeleton';
	
	import { getModalStore } from '@skeletonlabs/skeleton';
	import { RadioGroup, RadioItem } from '@skeletonlabs/skeleton';
	let value: number = 0;

	import type { ModalSettings } from '@skeletonlabs/skeleton';

    const modalStore = getModalStore();

    function openFormModal() {
        const modal: ModalSettings = {
            type: 'component',
            component: 'FormModal',
            title: 'Nová konfigurace  ewfacquire',
            response: (data: { name: string; email: string; message: string }) => {
                console.log('Formulář odeslán:', data);
                // Zde můžete přidat další logiku, např. odeslání dat na server
            }
        };
        modalStore.trigger(modal);
    }

	

	
</script>

<div class="container">
	<RadioGroup active="variant-filled-primary" hover="hover:variant-soft-primary">
		<RadioItem bind:group={value} name="justify" value={0}>DD</RadioItem>
		<RadioItem bind:group={value} name="justify" value={1}>ewfacquire</RadioItem>
	</RadioGroup>

	<button on:click={openFormModal} type="button" class="btn-icon variant-filled-primary">
		<div style="width: 15px;"><FaPlus /></div>
	</button>
</div>

<div class="config-grid">
	<section class="grid grid-cols-2 md:grid-cols-4 gap-4">
		{#each Array.from({ length: 12 }, (_, i) => i + 1) as item}
			<div class="grid-box">
				<div class="h-auto variant-ringed-primary max-w-full rounded-lg w-32 h-32">
					<div class="centered-container">
						<p class="name">Konf {item}</p>
						<p class="text">17.10.24 15:51</p>
						<p class="title">počet užití</p>
						<p class="text">{item * item}</p>
					</div>
				</div>
			</div>
		{/each}
	</section>
</div>

<style lang="postcss">
	.config-grid {
		padding-top: 20px;
		width: 50rem;
		margin: auto;
	}

	.centered-container {
		padding-top: 15px;
		padding-bottom: 5px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		text-align: center;
		height: 100%;
	}

	.centered-container .name {
		font-weight: bold;
		font-size: 1.5rem;
		color: #d4163c;
		padding-bottom: 10px;
	}

	.centered-container .title {
		font-weight: bold;
		font-size: 1rem;
		color: #d4163c;
	}

	.centered-container .text {
		font-size: 1rem;
		color: white;
	}

	.grid-box {
		display: flex;
		justify-content: center;
		align-items: center;
	}
	.container {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
	}
</style>
