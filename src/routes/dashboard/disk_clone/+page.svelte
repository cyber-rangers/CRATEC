<script lang="ts">
	import { Usb, HardDrive } from 'lucide-svelte';
	import { copyRunStore } from '$lib/stores/copyRunStore';
	import { deviceStore } from '$lib/stores/deviceStore';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import DiskSelectModal from '$lib/components/modals/DiskSelectModal.svelte';
	import { Slider } from '@skeletonlabs/skeleton-svelte';
	import { writable, type Writable } from 'svelte/store';
	import type { EwfParams } from '$lib/stores/copyRunStore';

	// Definice výchozího stavu copyRunStore – aby se resetoval při načtení stránky
	const defaultCopyRunState = {
		inputDisk: null,
		outputDisks: [],
		ewfParams: {
			case_number: '',
			description: '',
			investigator_name: '',
			evidence_number: '',
			notes: '',
			bytes_to_read: [0],
			offset: [0]
		},
		ddParams: {
			case_number: '',
			description: '',
			investigator_name: '',
			evidence_number: ''
		}
	};

	// Defaultní kroky (pro DD, pokud nejsou dynamické)
	const defaultSteps = [
		{ label: 'Step 1', description: 'The description of step 1.' },
		{ label: 'Step 2', description: 'The description of step 2.' },
		{ label: 'Step 3', description: 'The description of step 3.' },
		{ label: 'Step 4', description: 'The description of step 4.' },
		{ label: 'Step 5', description: 'The description of step 5.' }
	];

	type EwfParamsKeys = keyof EwfParams;
	// => 'case_number' | 'description' | 'investigator_name' | 'evidence_number' | 'notes' | 'bytes_to_read'

	interface StepDefinition {
		label: string;
		description: string;
		field?: EwfParamsKeys; // nepovinné, protože diskSelection krok žádný field nepotřebuje
		notesStep?: boolean;
		bytesStep?: boolean;
		rangeStep?: boolean;
		offsetStep?: boolean;
		// cokoliv dalšího
	}

	let DiskSelectModalOpen = false;
	let modalSide: 'input' | 'output' = 'input';
	let tabSet: number = 0;
	// Přepínač: 0 = ewfacquire, 1 = DD
	let value: number = 0;
	// Konfigurace získané z DB – objekt obsahující pole dd a ewf
	let configs: any = { dd: [], ewf: [] };

	async function loadConfigs() {
		try {
			configs = await invoke('get_all_active_configs');
			console.log('Loaded configs:', configs);
		} catch (error) {
			console.error('Error fetching configs:', error);
		}
	}

	// Při načtení stránky resetujeme copyRunStore, aby nebyly uloženy staré hodnoty
	onMount(async () => {
		copyRunStore.set(defaultCopyRunState);
		await loadConfigs();
	});

	// Stav výběru konfigurace
	let configSelected = false;
	let selectedConfig: any = null;

	function selectConfig(config: any) {
		selectedConfig = config;
		// Pokud má konfigurace atribut ewf_format, jedná se o ewfacquire, jinak DD
		value = config.ewf_format ? 0 : 1;
		configSelected = true;
		currentStep = 0;
	}

	$: maxBytes = (() => {
		const st = $copyRunStore;
		if (!st.inputDisk?.sector_count || !st.inputDisk?.sector_size) return 100;
		return st.inputDisk.sector_count * st.inputDisk.sector_size;
	})();

	let stepsToUse: StepDefinition[] = [];

	// Správa kroků
	let currentStep = 0;
	$: dynamicSteps =
		configSelected && value === 0 && selectedConfig
			? ([
					{ label: 'Disk Selection', description: 'Vyberte disky' },
					{ label: 'Case Number', description: 'Zadejte číslo případu', field: 'case_number' },
					{ label: 'Description', description: 'Zadejte popis', field: 'description' },
					{
						label: 'Investigator Name',
						description: 'Zadejte jméno vyšetřovatele',
						field: 'investigator_name'
					},
					{
						label: 'Evidence Number',
						description: 'Zadejte číslo důkazu',
						field: 'evidence_number'
					},
					...(selectedConfig.notes === 'ask'
						? [{ label: 'Notes', description: 'Zadejte poznámky', field: 'notes', notesStep: true }]
						: []),
					...(selectedConfig.offset === 'ask' && selectedConfig.bytes_to_read === 'ask'
						? [
								{
									label: 'Offset + Bytes to read',
									description: 'Zadejte ofset a kolik bajtů číst',
									rangeStep: true
								}
							]
						: selectedConfig.offset === 'ask'
							? [
									{
										label: 'Offset (start position)',
										description: 'Zadejte ofset (odkud začít číst)',
										offsetStep: true
									}
								]
							: selectedConfig.bytes_to_read === 'ask'
								? [
										{
											label: 'Bytes to read',
											description: 'Kolik bajtů číst',
											bytesStep: true
										}
									]
								: [])
				] as StepDefinition[])
			: [];

	$: stepsToUse = configSelected && value === 0 && selectedConfig ? dynamicSteps : defaultSteps;
	$: isFirstStep = currentStep === 0;
	$: isLastStep = currentStep === stepsToUse.length - 1;

	let bytes_slider: Writable<number[]> = writable<number[]>([0]);

	function isCurrentStep(index: number) {
		return currentStep === index;
	}
	function setStep(index: number) {
		currentStep = index;
	}
	function prevStep() {
		if (currentStep > 0) currentStep--;
	}

	async function runEwfAcquire() {
		try {
			const config_id = selectedConfig.id;
			const input_interface = $copyRunStore.inputDisk?.interface || '';
			// Vezmeme všechny výstupní disky (ne jen první):
			const output_interfaces = $copyRunStore.outputDisks.map(d => d.interface);

			const {
				case_number,
				description,
				investigator_name,
				evidence_number,
				notes,
				offset,
				bytes_to_read
			} = $copyRunStore.ewfParams;

			const ewf_params = {
				case_number,
				description,
				investigator_name,
				evidence_number,
				notes,
				offset: offset[0],
				bytes_to_read: bytes_to_read[0]
			};

			console.log('Volám run_ewfacquire s:', {
				config_id,
				ewf_params,
				input_interface,
				output_interfaces
			});

			await invoke('run_ewfacquire', {
				config_id,
				ewf_params,
				input_interface,
				output_interfaces
			});
			console.log('Ewfacquire dokončeno.');
		} catch (error) {
			console.error('Chyba při spouštění ewfacquire:', error);
		}
	}
	let processStarted = false;

	function nextStep() {
		if (!isLastStep) {
			currentStep++;
		} else {
			if (!processStarted) {
				processStarted = true;
				console.log('Final copyRunStore:', $copyRunStore);
				runEwfAcquire();
			}
		}
	}
</script>

{#if !configSelected}
	<!-- Zobrazení pouze výběru konfigurací -->
	<div class="conf-container space-y-2 text-center">
		<div style="padding-bottom: 20px;">
			<nav class="btn-group preset-outlined-surface-200-800 flex-col p-2 md:flex-row">
				<button
					type="button"
					onclick={() => (value = 0)}
					class="btn {value === 0 ? 'preset-filled' : 'hover:preset-tonal'}"
				>
					ewfacquire
				</button>
				<button
					type="button"
					onclick={() => (value = 1)}
					class="btn {value === 1 ? 'preset-filled' : 'hover:preset-tonal'}"
				>
					DD
				</button>
			</nav>
		</div>
		<section class="mx-auto grid w-5/6 grid-cols-2 gap-4 md:grid-cols-4">
			{#if value === 0}
				{#each configs.ewf as config (config.id)}
					<div class="grid-box">
						<div class="variant-ringed-primary"></div>
						<button class="centered-container" onclick={() => selectConfig(config)}>
							<p class="name">{config.confname}</p>
							<p class="text">ID: {config.id}</p>
							<p class="title">Formát</p>
							<p class="text">{config.ewf_format}</p>
						</button>
					</div>
				{/each}
			{:else if value === 1}
				{#each configs.dd as config (config.id)}
					<div class="grid-box">
						<div class="variant-ringed-primary"></div>
						<button class="centered-container" onclick={() => selectConfig(config)}>
							<p class="name">{config.confname}</p>
							<p class="text">ID: {config.id}</p>
							<p class="title">Block size</p>
							<p class="text">{config.bs} b</p>
						</button>
					</div>
				{/each}
			{/if}
		</section>
	</div>
{:else}
	<!-- Zobrazení stepperu a jednotlivých kroků -->
	<div class="w-full">
		<div class="space-y-8">
			<div class="relative">
				<div class="flex items-center justify-between gap-4">
					{#each stepsToUse as step, i}
						<button
							class="btn-icon btn-icon-sm rounded-full {isCurrentStep(i)
								? 'preset-filled-primary-500'
								: 'preset-filled-surface-200-800'}"
							onclick={() => setStep(i)}
							title={step.label}
						>
							<span class="font-bold">{i + 1}</span>
						</button>
					{/each}
				</div>
				<hr class="hr !border-surface-200-800 absolute top-[50%] right-0 left-0 z-[-1]" />
			</div>
			{#if currentStep === 0}
				<!-- Krok 0: Výběr disků -->
				<div class="card bg-surface-700-900 space-y-2 p-10 text-center" style="height: 373px;">
					<div class="container">
						<!-- Levá sekce (VSTUP) -->
						<div class="section left">
							<div class="header">VSTUP</div>
							<div class="content">
								{#if $copyRunStore.inputDisk}
									<button
										class="box"
										type="button"
										onclick={() => {
											modalSide = 'input';
											DiskSelectModalOpen = true;
										}}
									>
										{#if $copyRunStore.inputDisk.type === 'usb'}
											<div class="connected-icon" style="width: 40px;">
												<Usb />
											</div>
										{:else}
											<div class="connected-icon" style="width: 40px;">
												<HardDrive />
											</div>
										{/if}
										<span style="font-size:large;">{$copyRunStore.inputDisk.name}</span>
									</button>
								{:else}
									<button
										class="box"
										type="button"
										onclick={() => {
											modalSide = 'input';
											DiskSelectModalOpen = true;
										}}
									>
										+
									</button>
								{/if}
							</div>
						</div>
						<!-- Střední sekce (animovaná šipka) -->
						<div class="section center">
							<div class="content">
								<div class="arrow">
									<span></span>
									<span></span>
									<span></span>
								</div>
							</div>
						</div>
						<!-- Pravá sekce (VÝSTUP) -->
						<div class="section right">
							<div class="header">VÝSTUP</div>
							<div class="content">
								<div class="output-list">
									{#each $copyRunStore.outputDisks as disk (disk.interface)}
										<div class="output-item">
											{#if disk.type === 'usb'}
												<div class="connected-icon" style="width: 40px;">
													<Usb />
												</div>
											{:else}
												<div class="connected-icon" style="width: 40px;">
													<HardDrive />
												</div>
											{/if}
											<span style="font-size:large; font-weight:700;">{disk.name}</span>
										</div>
									{/each}
									{#if $copyRunStore.outputDisks.length < 2}
										<button
											class="box small"
											type="button"
											onclick={() => {
												modalSide = 'output';
												DiskSelectModalOpen = true;
											}}
										>
											+
										</button>
									{/if}
								</div>
							</div>
						</div>
					</div>
				</div>
			{:else}
				<div class="card bg-surface-700-900 space-y-2 p-10 text-center" style="height: 373px;">
					<h2 class="text-xl font-bold">{stepsToUse[currentStep].label}</h2>
					<p>{stepsToUse[currentStep].description}</p>

					{#if stepsToUse[currentStep].notesStep}
						<!-- Zadání poznámek -->
						<input
							type="text"
							bind:value={$copyRunStore.ewfParams[stepsToUse[currentStep].field as keyof EwfParams]}
							class="input w-[400px] block mx-auto text-center border-2 border-primary-500"
						/>
					{:else if stepsToUse[currentStep].rangeStep}
						<!-- Dva handly: [offset, offset + bytes_to_read] -->
						<!-- V skeleton-svelte: `<Slider range={true} ... />` pro dvou-handlový posun. -->
						<Slider
							
							value={[$copyRunStore.ewfParams.offset[0], $copyRunStore.ewfParams.bytes_to_read[0]]}
							max={maxBytes}
							onValueChange={(event) => {
								const [offsetVal, bytesVal] = event.value;
								copyRunStore.update((state) => ({
									...state,
									ewfParams: {
										...state.ewfParams,
										offset: [offsetVal],
										bytes_to_read: [bytesVal]
									}
								}));
							}}
						/>
						<p class="mt-2">
							Offset: {$copyRunStore.ewfParams.offset} / Bytes: {$copyRunStore.ewfParams
								.bytes_to_read}
						</p>
					{:else if stepsToUse[currentStep].offsetStep}
						<!-- Jedno-handlový slider, posun je offset. Vzorek s dir="rtl". -->
						<Slider
							dir="rtl"
							value={$copyRunStore.ewfParams.offset}
							max={maxBytes}
							onValueChange={(event) => {
								copyRunStore.update((state) => ({
									...state,
									ewfParams: {
										...state.ewfParams,
										offset: event.value
									}
								}));
							}}
						/>
						<p class="mt-2">Offset: {$copyRunStore.ewfParams.offset}</p>
					{:else if stepsToUse[currentStep].bytesStep}
						<!-- Stávající jednodeskový slider pro bytes_to_read -->
						<Slider
							value={$copyRunStore.ewfParams.bytes_to_read}
							max={maxBytes}
							markers={[25, 50, 75]}
							onValueChange={(event) => {
								copyRunStore.update((state) => {
									return {
										...state,
										ewfParams: {
											...state.ewfParams,
											bytes_to_read: event.value
										}
									};
								});
							}}
						/>
						<p class="mt-2">Zvoleno bajtů: {$copyRunStore.ewfParams.bytes_to_read}</p>
					{:else if stepsToUse[currentStep].field}
						<!-- Obecné textové pole pro "case_number", "description", "investigator_name", "evidence_number" atd. -->
						<input
							type="text"
							bind:value={$copyRunStore.ewfParams[stepsToUse[currentStep].field as keyof EwfParams]}
							class="input w-[400px] block mx-auto text-center border-2 border-primary-500"
						/>
					{/if}
				</div>
			{/if}
			<nav class="flex items-center justify-between gap-4">
				<button
					type="button"
					class="btn bg-surface-800"
					onclick={prevStep}
					disabled={isFirstStep}
				>
					<span>Předchozí</span>
				</button>
				<button
					type="button"
					class="btn bg-surface-800"
					onclick={nextStep}
					disabled={isLastStep && false}
				>
					<span>{isLastStep ? 'Dokončit' : 'Další'}</span>
				</button>
			</nav>
		</div>
	</div>
{/if}

<DiskSelectModal bind:openState={DiskSelectModalOpen} side={modalSide} />

<style lang="postcss">
	/* Stylování zůstává stejné, viz původní kód */
	.container {
		display: flex;
		width: 100%;
		height: 100%;
		overflow: hidden;
	}
	.section {
		flex: 1;
		position: relative;
		display: flex;
		flex-direction: column;
	}
	.header {
		position: absolute;
		top: 1rem;
		left: 0;
		right: 0;
		text-align: center;
		font-size: 1.5rem;
		font-weight: bold;
		z-index: 1;
		color: white;
	}
	.content {
		flex: 1;
		display: flex;
		justify-content: center;
		align-items: center;
	}
	.box {
		width: 120px;
		height: 120px;
		border: 3px solid var(--color-primary-500);
		border-radius: 15px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		font-size: 2rem;
		font-weight: bold;
		background: none;
		cursor: pointer;
		outline: none;
		color: white;
	}
	.box.small {
		width: 60px;
		height: 60px;
		font-size: 1.5rem;
	}
	.box span {
		font-size: 0.8rem;
		margin-top: 0.2rem;
	}
	.output-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		align-items: center;
	}
	.output-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		width: 170px;
		height: 70px;
		padding: 0.5rem;
		border: 3px solid var(--color-primary-500);
		border-radius: 15px;
		font-size: 0.9rem;
		color: white;
	}
	.arrow {
		position: absolute;
		top: 50%;
		left: 35%;
		transform: translate(-50%, -50%) rotate(270deg);
	}
	.arrow span {
		display: block;
		width: 5vw;
		height: 5vw;
		border-bottom: 15px solid var(--color-primary-500);
		border-right: 15px solid var(--color-primary-500);
		transform: rotate(45deg);
		margin: -10px;
		animation: animate 2s infinite;
	}
	.arrow span:nth-child(2) {
		animation-delay: -0.2s;
	}
	.arrow span:nth-child(3) {
		animation-delay: -0.4s;
	}
	@keyframes animate {
		0% {
			opacity: 0;
			transform: rotate(45deg) translate(-20px, -20px);
		}
		50% {
			opacity: 1;
		}
		100% {
			opacity: 0;
			transform: rotate(45deg) translate(20px, 20px);
		}
	}
	.centered-container {
		background-color: var(--color-surface-800);
		padding-top: 15px;
		padding-bottom: 5px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		text-align: center;
		height: 100%;
		border-radius: 22px;
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
	.conf-container {
		height: 495px;
		overflow-y: auto;
	}
	.conf-container::-webkit-scrollbar {
		width: 1px;
		background-color: transparent;
	}
	.variant-ringed-primary {
		position: absolute;
		width: 100%;
		height: 100%;
		border: 4px solid var(--color-primary-500);
		border-radius: 0.5rem;
		z-index: 2;
	}
	.centered-container {
		position: absolute;
		width: 100%;
		height: 100%;
		background-color: var(--color-surface-900);
		padding-top: 15px;
		padding-bottom: 5px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		text-align: center;
		border-radius: 28px;
		z-index: 3;
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
		position: relative;
		width: 8rem;
		height: 8rem;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
	}
	.no-disks-info {
		color: var(--color-primary-500);
		font-weight: bold;
		text-align: center;
		padding: 2rem;
	}
	/* Přizpůsobte kartu, aby měla maximálně výšku viewportu minus nějaké odsazení, 
   a současně nastavte overflow na hidden, aby se scroll neobjevil */
	.card {
		height: auto;
		max-height: 100vh; /* odečtete potřebné odsazení */
		overflow: hidden;
	}
</style>
