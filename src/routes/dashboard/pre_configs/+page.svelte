<script lang="ts">
	import { FaPlus } from "svelte-icons/fa";
	import {
		getModalStore,
		getToastStore,
		popup,
	} from "@skeletonlabs/skeleton";
	import { RadioGroup, RadioItem } from "@skeletonlabs/skeleton";
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import type {
		ModalSettings,
		ToastSettings,
		PopupSettings,
	} from "@skeletonlabs/skeleton";

	const modalStore = getModalStore();
	const toastStore = getToastStore();
	let value: number = 0; // 0 = DD, 1 = ewfacquire
	let configs: any[] = [];
	let popupSettings: Record<number, PopupSettings> = {};

	// Načte konfigurace z databáze
	async function loadConfigs() {
		try {
			configs = await invoke("get_all_ewf_configs");
			console.log("Loaded configs:", configs);

			// Dynamicky vytvoříme popup settings pro každý config
			popupSettings = configs.reduce(
				(acc, config) => {
					acc[config.id] = {
						event: "click",
						target: `config-popup-${config.id}`,
						placement: "bottom",
					};
					return acc;
				},
				{} as Record<number, PopupSettings>,
			);
		} catch (error) {
			console.error("Error fetching configs:", error);
		}
	}

	// Otevře správný modal na základě vybraného typu (DD nebo ewfacquire)
	function openFormModal() {
		const modal: ModalSettings = {
			type: "component",
			component: value === 0 ? "NewConfigDDModal" : "NewConfigEwfModal",
			title: value === 0 ? "Nová konfigurace DD" : "Nová konfigurace ewfacquire",
			response: async () => {
				await loadConfigs(); // Aktualizace seznamu po vytvoření
			},
		};
		modalStore.trigger(modal);
	}

	// Otevře modal s detaily vybrané konfigurace
	function openConfigModal(config: any) {
		console.log("Otevírám detail modal s config:", config);
		const modal: ModalSettings = {
			id: Date.now().toString(),
			type: "component",
			component: "EwfConfigDetailModal",
			meta: { configData: config },
			title: "Detaily konfigurace",
		};
		modalStore.trigger(modal);
	}

	// Funkce pro zobrazení potvrzovacího toastu a mazání konfigurace
	async function deleteConfig(configId: number) {
		const confirmToast: ToastSettings = {
			message: "Opravdu chcete odstranit tuto konfiguraci?",
			background: "variant-filled-primary",
			action: {
				label: "Smazat",
				response: async () => {
					try {
						await invoke("delete_or_deactivate_ewf_config", {
							config_id: configId,
						});
						console.log(`Konfigurace ${configId} byla smazána.`);
						toastStore.trigger({
							message: "Konfigurace byla odstraněna.",
							background: "variant-filled-success",
						});
						await loadConfigs(); // Aktualizace seznamu po smazání
					} catch (error) {
						console.error("Chyba při mazání konfigurace:", error);
						toastStore.trigger({
							message: "Nepodařilo se odstranit konfiguraci.",
							background: "variant-filled-error",
						});
					}
				},
			},
		};
		toastStore.trigger(confirmToast);
	}

	onMount(async () => {
		await loadConfigs();
	});
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
		{#each configs as config}
			<div class="grid-box">
				<div
					class="h-auto variant-ringed-primary max-w-full rounded-lg w-32 h-32"
					use:popup={popupSettings[config.id]}
				>
					<div class="centered-container">
						<p class="name">{config.confname}</p>
						<p class="text">ID: {config.id}</p>
						<p class="title">Formát</p>
						<p class="text">{config.ewf_format}</p>
					</div>
				</div>
			</div>

			<!-- Popup s možnostmi -->
			<div class="popup-menu card p-2 shadow-lg" data-popup={`config-popup-${config.id}`}>
				<button class="btn w-full text-left" on:click={() => openConfigModal(config)}>Info</button>
				<button class="btn w-full text-left text-red-500" on:click={() => deleteConfig(config.id)}>Smazat</button>
				<div class="arrow bg-surface-100-800-token" />
			</div>
		{/each}
	</section>
</div>

<style lang="postcss">
	.container {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
	}

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
		cursor: pointer;
	}

	.popup-menu {
		position: absolute;
		background: white;
		border-radius: 8px;
		box-shadow: 0px 4px 10px rgba(0, 0, 0, 0.1);
		width: 140px;
	}

	.popup-menu button {
		padding: 10px;
		font-size: 14px;
	}

	.popup-menu .arrow {
		position: absolute;
		width: 10px;
		height: 10px;
		background: white;
		transform: rotate(45deg);
		top: -5px;
		left: 50%;
		margin-left: -5px;
	}
</style>
