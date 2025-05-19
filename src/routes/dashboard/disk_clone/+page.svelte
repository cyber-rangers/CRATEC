<script lang="ts">
	import { Usb, HardDrive } from 'lucide-svelte';
	import { copyRunStore } from '$lib/stores/copyRunStore';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import DiskSelectModal from '$lib/components/modals/DiskSelectModal.svelte';
	import WarningModal from '$lib/components/modals/WarningModal.svelte';
	import { Slider } from '@skeletonlabs/skeleton-svelte';
	import type { EwfParams, DdParams } from '$lib/stores/copyRunStore';
	import VirtualKeyboard from '$lib/components/VirtualKeyboard.svelte';
	import { Toaster, createToaster } from '@skeletonlabs/skeleton-svelte';

	const toaster = createToaster({
		placement: 'top'
	});

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
			evidence_number: '',
			notes: '',
			limit: [0],
			offset: [0]
		}
	};

	const defaultSteps = [
		{ label: 'Step 1', description: 'The description of step 1.' },
		{ label: 'Step 2', description: 'The description of step 2.' },
		{ label: 'Step 3', description: 'The description of step 3.' },
		{ label: 'Step 4', description: 'The description of step 4.' },
		{ label: 'Step 5', description: 'The description of step 5.' }
	];

	type ParamKeys = keyof EwfParams | keyof DdParams;
	interface StepDefinition {
		label: string;
		description: string;
		field?: ParamKeys;
		notesStep?: boolean;
		bytesStep?: boolean;
		offsetStep?: boolean;
		rangeStep?: boolean;
	}

	let DiskSelectModalOpen = false;
	let modalSide: 'input' | 'output' = 'input';
	let tabSet: number = 0;
	let value: number = 0;
	let configs: any = { dd: [], ewf: [] };
	let configSelected = false;
	let selectedConfig: any = null;
	let currentStep = 0;

	let WarningModalOpen = false;

	let showKeyboard = false;
	let activeInput = '';
	let formData = {};

	let warningData = { dco: 0, has_hpa: false, readable: true };

	function openKeyboard(inputName: string) {
		activeInput = inputName;
		showKeyboard = true;
	}

	function closeKeyboard() {
		showKeyboard = false;
		activeInput = '';
	}

	function handleKeyboardInput(field: string, inputValue: string) {
		// value je proměnná určující typ konfigurace (0 = EWF, 1 = DD)
		if (value === 0) {
			// EWF
			copyRunStore.update((state) => {
				if (field in state.ewfParams) {
					(state.ewfParams as any)[field] = inputValue;
				}
				return state;
			});
		} else {
			// DD
			copyRunStore.update((state) => {
				if (field in state.ddParams) {
					(state.ddParams as any)[field] = inputValue;
				}
				return state;
			});
		}
	}

	async function loadConfigs() {
		try {
			configs = await invoke('get_all_active_configs');
			console.log('Loaded configs:', configs);
		} catch (error) {
			console.error('Error fetching configs:', error);
		}
	}

	onMount(async () => {
		copyRunStore.set(defaultCopyRunState);
		await loadConfigs();
	});

	function selectConfig(config: any) {
		selectedConfig = config;
		value = config.ewf_format ? 0 : 1;
		configSelected = true;
		currentStep = 0;
	}

	$: sectorCount = $copyRunStore.inputDisk?.sector_count || 100;
	$: sectorSize = $copyRunStore.inputDisk?.sector_size || 512;

	$: maxBytes = (() => {
		const st = $copyRunStore;
		return st.inputDisk?.sector_count && st.inputDisk?.sector_size
			? st.inputDisk.sector_count * st.inputDisk.sector_size
			: 100;
	})();

	$: stepsToUse =
		configSelected && selectedConfig
			? value === 0
				? [
						{ label: 'Výběr disku', description: 'Vyberte disky' },
						{
							label: 'Číslo případu',
							description: 'Zadejte číslo případu (Case number)',
							field: 'case_number'
						},
						{ label: 'Popis', description: 'Zadejte popis (Description)', field: 'description' },
						{
							label: 'Vyšetřovatel',
							description: 'Zadejte jméno vyšetřovatele (Investigator name)',
							field: 'investigator_name'
						},
						{
							label: 'Číslo důkazu',
							description: 'Zadejte číslo důkazu (Evidence number)',
							field: 'evidence_number'
						},
						...(selectedConfig.notes === 'ask'
							? [
									{
										label: 'Poznámky',
										description: 'Zadejte poznámky (Notes)',
										field: 'notes',
										notesStep: true
									}
								]
							: []),
						...(selectedConfig.offset === 'ask' && selectedConfig.bytes_to_read === 'ask'
							? [
									{
										label: 'Offset + Bytes',
										description: 'Zadejte offset a kolik bajtů číst (Offset + Bytes to read)',
										rangeStep: true
									}
								]
							: selectedConfig.offset === 'ask'
								? [{ label: 'Offset', description: 'Zadejte offset (Offset)', offsetStep: true }]
								: selectedConfig.bytes_to_read === 'ask'
									? [
											{
												label: 'Bytes to read',
												description: 'Kolik bajtů číst (Bytes to read)',
												bytesStep: true
											}
										]
									: [])
					]
				: [
						{ label: 'Výběr disku', description: 'Vyberte disky' },
						{
							label: 'Číslo případu',
							description: 'Zadejte číslo případu (Case number)',
							field: 'case_number'
						},
						{ label: 'Popis', description: 'Zadejte popis (Description)', field: 'description' },
						{
							label: 'Vyšetřovatel',
							description: 'Zadejte jméno vyšetřovatele (Investigator name)',
							field: 'investigator_name'
						},
						{
							label: 'Číslo důkazu',
							description: 'Zadejte číslo důkazu (Evidence number)',
							field: 'evidence_number'
						},
						...(selectedConfig.notes === 'ask'
							? [
									{
										label: 'Poznámky',
										description: 'Zadejte poznámky (Notes)',
										field: 'notes',
										notesStep: true
									}
								]
							: []),
						...(selectedConfig.offset === 'ask' && selectedConfig.limit_mode === 'ask'
							? [
									{
										label: 'Offset + Limit',
										description: 'Zadejte offset a limit (Offset + Limit)',
										rangeStep: true
									}
								]
							: selectedConfig.offset === 'ask'
								? [{ label: 'Offset', description: 'Zadejte offset (Offset)', offsetStep: true }]
								: selectedConfig.limit_mode === 'ask'
									? [{ label: 'Limit', description: 'Zadejte limit (Limit)', bytesStep: true }]
									: [])
					]
			: defaultSteps;

	$: isFirstStep = currentStep === 0;
	$: isLastStep = currentStep === stepsToUse.length - 1;

	$: canProceed = $copyRunStore.inputDisk !== null && $copyRunStore.outputDisks.length > 0;

	function isCurrentStep(index: number) {
		return currentStep === index;
	}
	function setStep(index: number) {
		currentStep = index;
	}
	function prevStep() {
		if (currentStep > 0) currentStep--;
	}

	let processStarted = false;

	async function handleFinalStep() {
		if (processStarted) return;
		processStarted = true;

		const sourceDisk = $copyRunStore.inputDisk;
		if (!sourceDisk) {
			processStarted = false;
			return;
		}
		const diskPath = `/dev/disk/by-path/${sourceDisk.interface}`;
		try {
			const diskInfo = (await invoke('get_disk_info', { device: diskPath })) as any;
			
			if (diskInfo.dco !== 0 || diskInfo.has_hpa === true || diskInfo.readable === false) {
				warningData = {
					dco: diskInfo.dco,
					has_hpa: diskInfo.has_hpa,
					readable: diskInfo.readable
				};
				WarningModalOpen = true;
				processStarted = false;
				return;
			}

			if (value === 0) runEwfAcquire();
			else runDdAcquire();
		} catch (e) {
			console.error('Chyba při získávání disk info:', e);
			WarningModalOpen = true;
			processStarted = false;
		}
	}

	function handleWarningResult(shouldContinue: boolean) {
		if (shouldContinue) {
			if (value === 0) runEwfAcquire();
			else runDdAcquire();
		} else {
			configSelected = false;
			currentStep = 0;
			copyRunStore.set(defaultCopyRunState);
		}
	}

	function nextStep() {
		const checkFields = ['case_number', 'evidence_number'];
		let invalidField = '';

		if (
			stepsToUse[currentStep]?.field &&
			checkFields.includes(stepsToUse[currentStep].field as string)
		) {
			const field = stepsToUse[currentStep].field as string;
			const valueToCheck =
				value === 0
					? $copyRunStore.ewfParams[field as keyof EwfParams]
					: $copyRunStore.ddParams[field as keyof DdParams];

			if (typeof valueToCheck === 'string' && !/^[A-Za-z0-9_-]*$/.test(valueToCheck)) {
				invalidField = field;
			}
		}

		if (invalidField) {
			copyRunStore.update((state) => {
				if (value === 0) {
					(state.ewfParams as any)[invalidField] = '';
				} else {
					(state.ddParams as any)[invalidField] = '';
				}
				return state;
			});
			toaster.warning({
				title: 'Neplatný znak',
				description: 'Použijte pouze písmena, čísla, pomlčku nebo podtržítko.'
			});
			return;
		}

		if (!isLastStep) {
			currentStep++;
		} else {
			handleFinalStep();
		}
	}

	async function runEwfAcquire() {
		try {
			const config_id = selectedConfig.id;
			const input_interface = $copyRunStore.inputDisk?.interface || '';
			const output_interfaces = $copyRunStore.outputDisks.map((d) => d.interface);

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

			invoke('run_ewfacquire', {
				config_id,
				ewf_params,
				input_interface,
				output_interfaces
			})
				.then(() => {
					toaster.success({
						title: 'Proces spuštěn',
						description: 'ewfacquire byl úspěšně spuštěn.'
					});
				})
				.catch((error) => {
					console.error('Chyba při spouštění ewfacquire:', error);
					toaster.error({
						title: 'Chyba',
						description: 'Proces ewfacquire selhal při spuštění.'
					});
				});

			configSelected = false;
			processStarted = false;
			currentStep = 0;
			copyRunStore.set(defaultCopyRunState);
		} catch (error) {
			console.error('Chyba při spouštění ewfacquire:', error);
		}
	}

	async function runDdAcquire() {
		try {
			const config_id = selectedConfig.id;
			const input_interface = $copyRunStore.inputDisk?.interface || '';
			const output_interfaces = $copyRunStore.outputDisks.map((d) => d.interface);

			const { case_number, description, investigator_name, evidence_number, notes, offset, limit } =
				$copyRunStore.ddParams;

			const sectorSize = $copyRunStore.inputDisk?.sector_size || 512;

			const dd_params = {
				case_number,
				description,
				investigator_name,
				evidence_number,
				notes,
				offset: offset[0],
				limit: limit[0] * sectorSize
			};

			console.log('Running DD acquire with:', {
				config_id,
				dd_params,
				input_interface,
				output_interfaces
			});

			invoke('run_dcfldd', {
				config_id,
				dd_params,
				input_interface,
				output_interfaces
			})
				.then(() => {
					toaster.success({
						title: 'Proces spuštěn',
						description: 'dcfldd byl úspěšně spuštěn.'
					});
				})
				.catch((error) => {
					console.error('Error running ddacquire:', error);
					toaster.error({
						title: 'Chyba',
						description: 'Proces dcfldd selhal při spuštění.'
					});
				});

			configSelected = false;
			processStarted = false;
			currentStep = 0;
			copyRunStore.set(defaultCopyRunState);
		} catch (error) {
			console.error('Error running ddacquire:', error);
		}
	}
</script>

{#if !configSelected}
	<!-- Výběr konfigurace -->
	<div class="conf-container space-y-2 text-center">
		<div style="padding-bottom: 20px;">
			<nav class="btn-group preset-outlined-surface-200-800 flex-col p-2 md:flex-row">
				<button
					type="button"
					on:click={() => (value = 0)}
					class="btn {value === 0 ? 'preset-filled' : 'hover:preset-tonal'}"
				>
					ewfacquire
				</button>
				<button
					type="button"
					on:click={() => (value = 1)}
					class="btn {value === 1 ? 'preset-filled' : 'hover:preset-tonal'}"
				>
					DD
				</button>
			</nav>
		</div>
		<section class="mx-auto grid w-5/6 grid-cols-2 gap-4 md:grid-cols-4">
			{#if value === 0}
				<!-- EWF configy -->
				{#each configs.ewf as config (config.id)}
					<div class="grid-box">
						<div class="variant-ringed-primary"></div>
						<button class="centered-container" on:click={() => selectConfig(config)}>
							<p class="name">{config.confname}</p>
							<p class="text">ID: {config.id}</p>
							<p class="title">Formát</p>
							<p class="text">{config.ewf_format}</p>
						</button>
					</div>
				{/each}
			{:else if value === 1}
				<!-- DD configy -->
				{#each configs.dd as config (config.id)}
					<div class="grid-box">
						<div class="variant-ringed-primary"></div>
						<button class="centered-container" on:click={() => selectConfig(config)}>
							<p class="name">{config.confname}</p>
							<p class="text">ID: {config.id}</p>
							<p class="title">Block size</p>
							<p class="text">{config.format}</p>
						</button>
					</div>
				{/each}
			{/if}
		</section>
	</div>
{:else}
	<!-- Stepper -->
	<div class="w-full">
		<div class="space-y-8">
			<div class="relative">
				<div class="flex items-center justify-between gap-4">
					{#each stepsToUse as step, i}
						<button
							class="btn-icon btn-icon-sm rounded-full {isCurrentStep(i)
								? 'preset-filled-primary-500'
								: 'preset-filled-surface-200-800'}"
							on:click={() => setStep(i)}
							title={step.label}
						>
							<span class="font-bold">{i + 1}</span>
						</button>
					{/each}
				</div>
				<hr class="hr !border-surface-200-800 absolute top-[50%] right-0 left-0 z-[-1]" />
			</div>

			<!-- Obsah jednotlivých kroků -->
			{#if currentStep === 0}
				<!-- Výběr disků -->
				<div class="card bg-surface-700-900 space-y-2 p-10 text-center" style="height: 373px;">
					<div class="container">
						<div class="section left">
							<div class="header">VSTUP</div>
							<div class="content">
								{#if $copyRunStore.inputDisk}
									<button
										class="box"
										type="button"
										on:click={() => {
											modalSide = 'input';
											DiskSelectModalOpen = true;
										}}
									>
										{#if $copyRunStore.inputDisk.type === 'usb'}
											<div style="width: 40px;">
												<Usb />
											</div>
										{:else}
											<div style="width: 40px;">
												<HardDrive />
											</div>
										{/if}
										<span style="font-size:large;">{$copyRunStore.inputDisk.name}</span>
									</button>
								{:else}
									<button
										class="box"
										type="button"
										on:click={() => {
											modalSide = 'input';
											DiskSelectModalOpen = true;
										}}
									>
										+
									</button>
								{/if}
							</div>
						</div>
						<div class="section center">
							<div class="content">
								<div class="arrow">
									<span></span>
									<span></span>
									<span></span>
								</div>
							</div>
						</div>
						<div class="section right">
							<div class="header">VÝSTUP</div>
							<div class="content">
								<div class="output-list">
									{#each $copyRunStore.outputDisks as disk (disk.interface)}
										<div class="output-item">
											{#if disk.type === 'usb'}
												<div style="width: 40px;">
													<Usb />
												</div>
											{:else}
												<div style="width: 40px;">
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
											on:click={() => {
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
				<!-- Ostatní kroky -->
				<div class="card bg-surface-700-900 space-y-2 p-10 text-center" style="height: 373px;">
					<h2 class="text-xl font-bold">{stepsToUse[currentStep].label}</h2>
					<p>{stepsToUse[currentStep].description}</p>

					<!-- notesStep -->
					{#if stepsToUse[currentStep].notesStep}
						{#if value === 0}
							<input
								type="text"
								class="input border-primary-500 mx-auto block w-[400px] border-2 text-center"
								bind:value={$copyRunStore.ewfParams.notes}
								on:focus={() => openKeyboard('notes')}
							/>
						{:else}
							<input
								type="text"
								class="input border-primary-500 mx-auto block w-[400px] border-2 text-center"
								bind:value={$copyRunStore.ddParams.notes}
								on:focus={() => openKeyboard('notes')}
							/>
						{/if}

						<!-- rangeStep => offset + bytes/limit -->
					{:else if stepsToUse[currentStep].rangeStep}
						{#if value === 0}
							<!-- EWF: offset a bytes_to_read v sektorech -->
							<label>Rozsah (Offset + Bytes to read) [LBA sektory]</label>
							<Slider
								value={[
									$copyRunStore.ewfParams.offset[0],
									$copyRunStore.ewfParams.offset[0] +
										Math.floor($copyRunStore.ewfParams.bytes_to_read[0] / sectorSize)
								]}
								max={sectorCount}
								onValueChange={(event) => {
									const [start, end] = event.value;
									copyRunStore.update((state) => {
										state.ewfParams.offset = [start];
										state.ewfParams.bytes_to_read = [(end - start) * sectorSize];
										return state;
									});
								}}
							/>
							<p class="mt-2">
								Offset: {$copyRunStore.ewfParams.offset[0]} ({$copyRunStore.ewfParams.offset[0] *
									sectorSize} B), Bytes to read: {Math.floor(
									$copyRunStore.ewfParams.bytes_to_read[0] / sectorSize
								)} sektorů ({$copyRunStore.ewfParams.bytes_to_read[0]} B)
							</p>
						{:else}
							<!-- DD: offset a limit v sektorech -->
							<label>Rozsah (Offset + Limit) [LBA sektory]</label>
							<Slider
								value={[
									$copyRunStore.ddParams.offset[0],
									$copyRunStore.ddParams.offset[0] + $copyRunStore.ddParams.limit[0]
								]}
								max={sectorCount}
								onValueChange={(event) => {
									const [start, end] = event.value;
									copyRunStore.update((state) => {
										state.ddParams.offset = [start];
										state.ddParams.limit = [end - start];
										return state;
									});
								}}
							/>
							<p class="mt-2">
								Offset: {$copyRunStore.ddParams.offset[0]} sektorů ({$copyRunStore.ddParams
									.offset[0] * sectorSize} B), Limit: {$copyRunStore.ddParams.limit[0]} sektorů ({$copyRunStore
									.ddParams.limit[0] * sectorSize} B)
							</p>
						{/if}

						<!-- offsetStep (jedno slider) -->
					{:else if stepsToUse[currentStep].offsetStep}
						{#if value === 0}
							<Slider
								dir="rtl"
								value={$copyRunStore.ewfParams.offset}
								max={sectorCount}
								onValueChange={(event) => {
									copyRunStore.update((state) => {
										state.ewfParams.offset = event.value;
										return state;
									});
								}}
							/>
							<p class="mt-2">
								Offset: {$copyRunStore.ewfParams.offset[0]} sektorů ({$copyRunStore.ewfParams
									.offset[0] * sectorSize} B)
							</p>
						{:else}
							<Slider
								dir="rtl"
								value={$copyRunStore.ddParams.offset}
								max={sectorCount}
								onValueChange={(event) => {
									copyRunStore.update((state) => {
										state.ddParams.offset = event.value;
										return state;
									});
								}}
							/>
							<p class="mt-2">
								Offset: {$copyRunStore.ddParams.offset[0]} sektorů ({$copyRunStore.ddParams
									.offset[0] * sectorSize} B)
							</p>
						{/if}

						<!-- bytesStep (EWF) / limitStep (DD) -->
					{:else if stepsToUse[currentStep].bytesStep}
						{#if value === 0}
							<Slider
								value={[$copyRunStore.ewfParams.bytes_to_read[0] / sectorSize]}
								max={sectorCount}
								onValueChange={(event) => {
									copyRunStore.update((state) => {
										state.ewfParams.bytes_to_read = [event.value[0] * sectorSize];
										return state;
									});
								}}
							/>
							<p class="mt-2">
								Bytes to read: {Math.floor($copyRunStore.ewfParams.bytes_to_read[0] / sectorSize)} sektorů
								({$copyRunStore.ewfParams.bytes_to_read[0]} B)
							</p>
						{:else}
							<Slider
								value={$copyRunStore.ddParams.limit}
								max={sectorCount}
								onValueChange={(event) => {
									copyRunStore.update((state) => {
										state.ddParams.limit = event.value;
										return state;
									});
								}}
							/>
							<p class="mt-2">
								Limit: {$copyRunStore.ddParams.limit[0]} sektorů ({$copyRunStore.ddParams.limit[0] *
									sectorSize} B)
							</p>
						{/if}

						<!-- Obecný field (case_number, description, investigator_name, evidence_number apod.) -->
					{:else if stepsToUse[currentStep].field}
						<!-- Ověříme, že field existuje -->
						{#if value === 0}
							<input
								type="text"
								class="input border-primary-500 mx-auto block w-[400px] border-2 text-center"
								bind:value={
									$copyRunStore.ewfParams[stepsToUse[currentStep].field as keyof EwfParams]
								}
								on:focus={() => openKeyboard(stepsToUse[currentStep].field ?? '')}
							/>
						{:else}
							<input
								type="text"
								class="input border-primary-500 mx-auto block w-[400px] border-2 text-center"
								bind:value={$copyRunStore.ddParams[stepsToUse[currentStep].field as keyof DdParams]}
								on:focus={() => openKeyboard(stepsToUse[currentStep].field ?? '')}
							/>
						{/if}
					{/if}
				</div>
			{/if}

			<!-- Navigační tlačítka kroků -->
			<nav class="flex items-center justify-between gap-4">
				<button type="button" class="btn bg-surface-800" on:click={prevStep} disabled={isFirstStep}>
					<span>Předchozí</span>
				</button>
				<button
					type="button"
					class="btn bg-surface-800"
					on:click={isLastStep ? handleFinalStep : nextStep}
					disabled={!canProceed}
				>
					<span>{isLastStep ? 'Dokončit' : 'Další'}</span>
				</button>
			</nav>
		</div>
	</div>
{/if}

<DiskSelectModal bind:openState={DiskSelectModalOpen} side={modalSide} />

<WarningModal bind:openState={WarningModalOpen} {warningData} onResult={handleWarningResult} />

<VirtualKeyboard
	bind:showKeyboard
	bind:activeInput
	formData={$copyRunStore[value === 0 ? 'ewfParams' : 'ddParams'] as unknown as Record<
		string,
		string | boolean | string[] | number[]
	>}
	on:closeKeyboard={closeKeyboard}
	onInputChange={handleKeyboardInput}
/>

<Toaster {toaster} />

<style lang="postcss">
	/* Ponecháváme vaše původní styly: */
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
	.card {
		height: auto;
		max-height: 100vh;
		overflow: hidden;
	}
</style>
