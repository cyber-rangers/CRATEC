<script lang="ts">
	import { Usb, HardDrive } from 'lucide-svelte';
	import { copyRunStore } from '$lib/stores/copyRunStore';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import DiskSelectModal from '$lib/components/modals/DiskSelectModal.svelte';

	// Pro DD konfigurace ponecháme defaultní kroky (zatím)
	const defaultSteps = [
		{ label: 'Step 1', description: 'The description of step 1.' },
		{ label: 'Step 2', description: 'The description of step 2.' },
		{ label: 'Step 3', description: 'The description of step 3.' },
		{ label: 'Step 4', description: 'The description of step 4.' },
		{ label: 'Step 5', description: 'The description of step 5.' }
	];

	let DiskSelectModalOpen = false;

	let tabSet: number = 0;
	// Přepínač: 0 = DD, 1 = ewfacquire
	let value: number = 0;
	// Konfigurace získané z DB – objekt obsahující pole dd a ewf
	let configs: any = { dd: [], ewf: [] };
	let meta = { side: '' };

	async function loadConfigs() {
		try {
			configs = await invoke('get_all_active_configs');
			console.log('Loaded configs:', configs);
		} catch (error) {
			console.error('Error fetching configs:', error);
		}
	}

	onMount(async () => {
		await loadConfigs();
	});

	// Stav zobrazení: zda byla vybrána konfigurace a její údaje
	let configSelected = false;
	let selectedConfig: any = null;

	// Při výběru konfigurace uložíme vybraný config a nastavíme správnou hodnotu "value"
	function selectConfig(config: any) {
		selectedConfig = config;
		// Pokud má konfigurace atribut ewf_format, jedná se o ewfacquire
		if (config.ewf_format) {
			value = 1;
		} else {
			value = 0;
		}
		configSelected = true;
		currentStep = 0;
	}

	// Správa kroků – currentStep je index aktuálně zobrazeného kroku
	let currentStep = 0;

	// Dynamické sestavení kroků pro ewfacquire konfigurace:
	// Vždy přidáme krok "Disk Selection" a dále vždy kroky pro case_number, description, investigator_name, evidence_number.
	// Krok pro notes přidáme pouze pokud selectedConfig.notes === "ask".
	$: dynamicSteps =
		configSelected && value === 1 && selectedConfig
			? [
					{ label: 'Disk Selection', description: 'Vyberte disky' },
					{ label: 'Case Number', description: 'Zadejte číslo případu' },
					{ label: 'Description', description: 'Zadejte popis' },
					{ label: 'Investigator Name', description: 'Zadejte jméno vyšetřovatele' },
					{ label: 'Evidence Number', description: 'Zadejte číslo důkazu' },
					...(selectedConfig.notes === 'ask'
						? [{ label: 'Notes', description: 'Zadejte poznámky' }]
						: [])
				]
			: [];

	// Pokud je vybrána ewfacquire konfigurace, použijeme dynamické kroky, jinak defaultSteps
	$: stepsToUse = configSelected && value === 1 && selectedConfig ? dynamicSteps : defaultSteps;
	$: isFirstStep = currentStep === 0;
	$: isLastStep = currentStep === stepsToUse.length - 1;

	function isCurrentStep(index: number) {
		return currentStep === index;
	}
	function setStep(index: number) {
		currentStep = index;
	}
	function prevStep() {
		if (currentStep > 0) currentStep--;
	}
	function nextStep() {
		if (currentStep < stepsToUse.length - 1) currentStep++;
	}

    //TESTOVACIIIIIIIIIIIIIIIIIIIIIIIIIIIIII
    async function testEwfAcquisition() {
    const config_id = 1; // fake config id
    const ewf_params = {
      case_number: "TEST_CASE_001",
      description: "Test process for ewfacquire",
      investigator_name: "Agent Smith",
      evidence_number: "EVID123",
	  notes: "testovaci poznamky",
	  offset: 0,
	  bytes_to_read: 50000,
    };
    const input_interface = "/dev/sdc";
    const output_interface = "/run/media/master/USB2/jebinka";
    
    try {
      const result = await invoke("run_ewfacquire", {
        config_id,
        ewf_params,
        input_interface,
        output_interface,
      });
      console.log("Ewf acquisition started:", result);
    } catch (error) {
      console.error("Error starting ewfacquire:", error);
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
					DD
				</button>
				<button
					type="button"
					onclick={() => (value = 1)}
					class="btn {value === 1 ? 'preset-filled' : 'hover:preset-tonal'}"
				>
					ewfacquire
				</button>
			</nav>
		</div>
		<section class="mx-auto grid w-5/6 grid-cols-2 gap-4 md:grid-cols-4">
			{#if value === 0}
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
			{:else if value === 1}
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
			{/if}
		</section>
	</div>
{:else}
	<!-- Zobrazení stepperu a jednotlivých kroků -->
	<div class="w-full">
		<div class="space-y-8">
			<!-- Timeline – tlačítka pro jednotlivé kroky -->
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

			<!-- Obsah aktuálního kroku -->
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
											meta = { side: 'input' };
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
										<span style="font-size:large;">{$copyRunStore.inputDisk.interface}</span>
									</button>
								{:else}
									<button
										class="box"
										type="button"
										onclick={() => {
											meta = { side: 'input' };
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
											<span style="font-size:large; font-weight:700;">{disk.interface}</span>
										</div>
									{/each}
									<button
										class="box small"
										type="button"
										onclick={() => {
											meta = { side: 'output' };
											DiskSelectModalOpen = true;
										}}
									>
										+
									</button>
								</div>
							</div>
						</div>
					</div>
				</div>
			{:else}
				<!-- Další kroky pro ewfacquire: každý krok obsahuje titulek, popis a vstupní pole -->
				<div class="card bg-surface-700-900 space-y-2 p-10 text-center" style="height: 373px;">
					<h2 class="text-xl font-bold">{stepsToUse[currentStep].label}</h2>
					<p>{stepsToUse[currentStep].description}</p>
					<input
						type="text"
						placeholder={stepsToUse[currentStep].label}
						class="mt-4 rounded border p-2"
					/>
                    <button onclick={testEwfAcquisition}>
                        Test run ewfacquire
                      </button>
				</div>
			{/if}

			<!-- Navigační tlačítka pro krokování -->
			<nav class="flex items-center justify-between gap-4">
				<button
					type="button"
					class="hover:preset-filled btn variant-filled"
					onclick={prevStep}
					disabled={isFirstStep}
				>
					<span>Předchozí</span>
				</button>
				<button
					type="button"
					class="hover:preset-filled btn variant-filled"
					onclick={nextStep}
					disabled={isLastStep}
				>
					<span>Další</span>
				</button>
			</nav>
		</div>
	</div>
{/if}

<DiskSelectModal bind:openState={DiskSelectModalOpen} {meta} />

<style lang="postcss">
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
</style>
