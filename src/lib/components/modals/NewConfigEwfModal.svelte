<script lang="ts">
	import { Modal, Combobox, Popover } from '@skeletonlabs/skeleton-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import VirtualKeyboard from '../VirtualKeyboard.svelte';
	import { createEventDispatcher } from 'svelte';
	import { X, Info } from 'lucide-svelte';

	// Řízení zobrazení modalu a předání vlastností z rodiče
	export let open: boolean = false;
	export let parent: { buttonTextSubmit?: string } = {};
	const dispatch = createEventDispatcher();

	// Data formuláře s výchozími hodnotami
	let formData = {
		confname: '',
		codepage: 'ascii',
		sectors_per_read: '64',
		bytes_to_read: 'whole',
		compression_method: 'deflate',
		compression_level: 'none',
		hash_types: [],
		ewf_format: 'encase6',
		granularity_sectors: '2',
		notes: 'ask',
		offset: '0',
		process_buffer_size: 'auto',
		bytes_per_sector: 'auto',
		quiet_mode: false,
		read_retry_count: '2',
		swap_byte_pairs: false,
		segment_size: '1.4',         // Jenom numerická část
		segment_size_unit: 'GiB',    // Jednotka (GiB, MiB, TiB)
		verbose_output: false,
		zero_on_read_error: false,
		use_chunk_data: false,
		hashformat: 'hex',
		totalhashformat: 'hex',
		hashconv: 'before',
		diffwr: 'off',
		sizeprobe: 'if'
	};

	// Možnosti pro Comboboxy
	const sectorsPerReadOptions = [
		{ label: '16', value: '16' },
		{ label: '32', value: '32' },
		{ label: '64', value: '64' },
		{ label: '128', value: '128' },
		{ label: '256', value: '256' },
		{ label: '512', value: '512' },
		{ label: '1024', value: '1024' },
		{ label: '2048', value: '2048' },
		{ label: '4096', value: '4096' },
		{ label: '8192', value: '8192' },
		{ label: '16384', value: '16384' },
		{ label: '32768', value: '32768' }
	];

	const bytesToReadOptions = [
		{ label: 'celý disk (výchozí)', value: 'whole' },
		{ label: 'dotázat', value: 'ask' }
	];

	const compressionMethodOptions = [
		{ label: 'deflate', value: 'deflate' }
	];

	const compressionLevelOptions = [
		{ label: 'none (výchozí)', value: 'none' },
		{ label: 'empty-block', value: 'empty-block' },
		{ label: 'fast', value: 'fast' },
		{ label: 'best', value: 'best' }
	];

	const statusOptions = [
		{ label: 'Zapnuto', value: 'on' },
		{ label: 'Vypnuto', value: 'off' }
	];

	const splitOptions = [
		{ label: 'Zeptat se', value: 'ask' },
		{ label: 'Zakázáno', value: 'disabled' }
	];

	const hashformatOptions = [
		{ label: 'hex', value: 'hex' },
		{ label: 'base64', value: 'base64' }
	];

	const totalhashformatOptions = [
		{ label: 'hex', value: 'hex' },
		{ label: 'base64', value: 'base64' }
	];

	const diffwrOptions = [
		{ label: 'off', value: 'off' },
		{ label: 'on', value: 'on' }
	];

	const hashconvOptions = [
		{ label: 'before', value: 'before' },
		{ label: 'after', value: 'after' }
	];

	const sizeprobeOptions = [
		{ label: 'if (zdroj)', value: 'if' },
		{ label: 'of (výstup)', value: 'of' }
	];

	const processBufferSizeOptions = [
		{ label: 'Automaticky detekovat (výchozí)', value: 'auto' },
		{ label: '128', value: '128' },
		{ label: '256', value: '256' },
		{ label: '512', value: '512' },
		{ label: '1024', value: '1024' },
		{ label: '2048', value: '2048' },
		{ label: '4096', value: '4096' },
		{ label: '8192', value: '8192' },
		{ label: '16384', value: '16384' }
	];

	const bytesPerSectorOptions = [
		{ label: 'Automaticky detekovat (výchozí)', value: 'auto' },
		{ label: '128', value: '128' },
		{ label: '256', value: '256' },
		{ label: '512', value: '512' },
		{ label: '1024', value: '1024' },
		{ label: '2048', value: '2048' },
		{ label: '4096', value: '4096' },
		{ label: '8192', value: '8192' },
		{ label: '16384', value: '16384' }
	];

	const notesOptions = [
		{ label: 'dotázat (výchozí)', value: 'ask' },
		{ label: 'neuvádět', value: 'empty' }
	];

	// Řídící proměnná pro aktivní Popover
	let activePopover: string | null = null;

	// Zobrazení virtuální klávesnice
	let showKeyboard = false;
	let activeInput: string = '';

	// Odeslání formuláře -> voláme Rust funkci
	async function onFormSubmit(): Promise<void> {
		try {
			// Kompilace velikosti segmentu do "1.4 GiB" (pokud to takto potřebujete v Rustu)
			const finalSegmentSize = `${formData.segment_size} ${formData.segment_size_unit}`;
			formData.segment_size = finalSegmentSize;

			const result = await invoke('save_new_ewf_config', formData);
			console.log('Formulář odeslán a data uložena, výsledek:', result);

			dispatch('response', formData);
			dispatch('close');

			// Po odeslání: schovat klávesnici a zavřít popovery
			showKeyboard = false;
			activePopover = null;
		} catch (error) {
			console.error('Chyba při odesílání formuláře:', error);
		}
	}

	// Když se focusne input -> aktivovat klávesnici
	function onInputFocus(event: Event, fieldName: string) {
		console.log('onInputFocus volá se pro:', fieldName);
		showKeyboard = true;
		activeInput = fieldName;
	}

	// Funkce s vysvětlením jednotlivých polí
	function getExplanation(field: string): string {
		switch (field) {
			case 'confname':
				return 'Název konfigurace slouží k identifikaci akvizice a je uveden v hlavičce výstupního souboru.';
			case 'codepage':
				return 'Určuje kódovou stránku pro interpretaci znaků v hlavičce. Možnosti: ascii (výchozí), windows-874, windows-932, až windows-1258.';
			case 'sectors_per_read':
				return 'Specifikuje počet sektorů, které se čtou najednou. Např. 64 znamená, že se najednou přečte 64 sektorů.';
			case 'bytes_to_read':
				return 'Počet bajtů k získání při každém čtení. "celý disk" znamená kompletní akvizici, "dotázat" umožňuje interaktivní volbu.';
			case 'compression_method':
				return 'Vybere se kompresní metoda. Výchozí je "deflate".';
			case 'compression_level':
				return 'Nastavuje úroveň komprese. Možnosti: none, empty-block, fast, best.';
			case 'status':
				return 'Určuje, zda se mají zobrazovat průběžné status zprávy během akvizice.';
			case 'split':
				return 'Řídí rozdělení souboru na segmenty. "ask" umožní volbu, "disabled" znamená, že se disk nerozdělí.';
			case 'hashformat':
				return 'Nastavuje formát průběžného hashe (např. hex/base64).';
			case 'totalhashformat':
				return 'Určuje formát celkového hashe (hex nebo base64).';
			case 'hashconv':
				return 'Definuje, zda se má hashování provádět před nebo po konverzi dat.';
			case 'diffwr':
				return 'Určuje, zda se mají zapisovat pouze změněné bloky (šetří místo).';
			case 'sizeprobe':
				return 'Nastavuje, zda se má velikost indikátoru počítat ze zdrojových nebo výstupních dat.';
			case 'process_buffer_size':
				return 'Definuje velikost bufferu pro zpracování dat. Výchozí = auto.';
			case 'bytes_per_sector':
				return 'Určuje počet bajtů na sektor; "auto" = automatická detekce, jinak konkrétní hodnota.';
			case 'notes':
				return 'Určuje, zda se mají do výstupu zahrnout poznámky. "ask" = interaktivní volba, "empty" = žádné poznámky.';
			case 'segment_size':
				return 'Určuje velikost segmentu souboru. Zadejte hodnotu a vyberte jednotku (GiB, MiB, TiB).';
			default:
				return 'Informace o tomto poli.';
		}
	}
</script>

<Modal
	{open}
	onOpenChange={(e) => {
		if (!e.open) {
			dispatch('close');
			showKeyboard = false;
			activePopover = null;
		}
	}}
	triggerBase="hidden"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-md"
	backdropClasses="backdrop-blur-sm"
>
	<!-- Trigger -->
	{#snippet trigger()}
		<span style="display: none;"></span>
	{/snippet}

	<!-- Content -->
	{#snippet content()}
		<div class="modal-form">
			<header class="modal-header">
				<h2>Nová konfigurace EWF</h2>
			</header>

			<form on:submit|preventDefault={onFormSubmit} class="modal-form-content">
				<!-- 1. Název konfigurace -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Název konfigurace</span>
						<Popover
							open={activePopover === 'confname'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'confname')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('confname')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<input
						class="input"
						name="confname"
						type="text"
						maxlength="12"
						bind:value={formData.confname}
						required
						on:focus={(e) => onInputFocus(e, 'confname')}
					/>
				</label>

				<!-- 2. Kódová stránka -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Kódová stránka</span>
						<Popover
							open={activePopover === 'codepage'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'codepage')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('codepage')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<select
						class="select"
						name="codepage"
						bind:value={formData.codepage}
					>
						<option value="ascii">ascii (výchozí)</option>
						<option value="windows-874">windows-874</option>
						<option value="windows-932">windows-932</option>
						<option value="windows-936">windows-936</option>
						<option value="windows-949">windows-949</option>
						<option value="windows-950">windows-950</option>
						<option value="windows-1250">windows-1250</option>
						<option value="windows-1251">windows-1251</option>
						<option value="windows-1252">windows-1252</option>
						<option value="windows-1253">windows-1253</option>
						<option value="windows-1254">windows-1254</option>
						<option value="windows-1255">windows-1255</option>
						<option value="windows-1256">windows-1256</option>
						<option value="windows-1257">windows-1257</option>
						<option value="windows-1258">windows-1258</option>
					</select>
				</label>

				<!-- 3. Počet sektorů na čtení (-b) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Počet sektorů na čtení (-b)</span>
						<Popover
							open={activePopover === 'sectors_per_read'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'sectors_per_read')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">
										{getExplanation('sectors_per_read')}
									</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={sectorsPerReadOptions}
						defaultValue={[formData.sectors_per_read]}
						value={[formData.sectors_per_read]}
						onValueChange={(e) => (formData.sectors_per_read = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 4. Počet bajtů k získání (-B) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Počet bajtů k získání (-B)</span>
						<Popover
							open={activePopover === 'bytes_to_read'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'bytes_to_read')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">
										{getExplanation('bytes_to_read')}
									</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={bytesToReadOptions}
						defaultValue={[formData.bytes_to_read]}
						value={[formData.bytes_to_read]}
						onValueChange={(e) => (formData.bytes_to_read = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 5. Metoda komprese (-c) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Metoda komprese (-c)</span>
						<Popover
							open={activePopover === 'compression_method'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'compression_method')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">
										{getExplanation('compression_method')}
									</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={compressionMethodOptions}
						defaultValue={[formData.compression_method]}
						value={[formData.compression_method]}
						onValueChange={(e) => (formData.compression_method = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 6. Úroveň komprese (-c) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Úroveň komprese (-c)</span>
						<Popover
							open={activePopover === 'compression_level'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'compression_level')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">
										{getExplanation('compression_level')}
									</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={compressionLevelOptions}
						defaultValue={[formData.compression_level]}
						value={[formData.compression_level]}
						onValueChange={(e) => (formData.compression_level = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 7. Hashovací algoritmy (-d) – MD5 vždy zahrnuto -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Hashovací algoritmy (-d) – MD5 vždy zahrnuto</span>
						<Popover
							open={activePopover === 'hash_types'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'hash_types')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">
										Vyberte dodatečné hashovací algoritmy (SHA1, SHA256) kromě výchozího MD5.
									</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<div class="flex gap-4">
						<label class="flex items-center">
							<input
								type="checkbox"
								class="checkbox"
								bind:group={formData.hash_types}
								value="sha1"
							/>
							<span class="ml-2">SHA1</span>
						</label>
						<label class="flex items-center">
							<input
								type="checkbox"
								class="checkbox"
								bind:group={formData.hash_types}
								value="sha256"
							/>
							<span class="ml-2">SHA256</span>
						</label>
					</div>
				</label>

				<!-- 8. Počet bajtů na hash okno (-d) -->
				<label class="label">
					<span>Počet bajtů na hash okno (-d)</span>
					<input
						class="input"
						name="hashwindow"
						type="number"
						bind:value={formData.hashwindow}
						on:focus={(e) => onInputFocus(e, 'hashwindow')}
					/>
				</label>

				<!-- 9. Log soubor pro hash -->
				<label class="label">
					<span>Log soubor pro hash</span>
					<input
						class="input"
						name="hashlog"
						type="text"
						bind:value={formData.hashlog}
						on:focus={(e) => onInputFocus(e, 'hashlog')}
					/>
				</label>

				<!-- 10. Zobrazovat status zprávy -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Zobrazovat status zprávy</span>
						<Popover
							open={activePopover === 'status'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'status')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('status')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={statusOptions}
						defaultValue={[formData.status]}
						value={[formData.status]}
						onValueChange={(e) => (formData.status = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 11. Interval status zpráv (-statusinterval) -->
				<label class="label">
					<span>Interval status zpráv (-statusinterval)</span>
					<input
						class="input"
						name="statusinterval"
						type="number"
						bind:value={formData.statusinterval}
						on:focus={(e) => onInputFocus(e, 'statusinterval')}
					/>
				</label>

				<!-- 12. Rozdělení souborů po velikosti (split) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Rozdělení souborů po velikosti (split)</span>
						<Popover
							open={activePopover === 'split'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'split')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('split')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={splitOptions}
						defaultValue={[formData.split]}
						value={[formData.split]}
						onValueChange={(e) => (formData.split = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 13. Formát rozdělení souboru (splitformat) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Formát rozdělení souboru (splitformat)</span>
						<Popover
							open={activePopover === 'splitformat'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'splitformat')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">
										{getExplanation('splitformat')}
									</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<input
						class="input"
						name="splitformat"
						type="text"
						bind:value={formData.splitformat}
						on:focus={(e) => onInputFocus(e, 'splitformat')}
					/>
				</label>

				<!-- 14. Ověření souboru (vf) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Ověření souboru (vf)</span>
						<Popover
							open={activePopover === 'vf'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'vf')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('vf')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<input
						class="input"
						name="vf"
						type="text"
						bind:value={formData.vf}
						on:focus={(e) => onInputFocus(e, 'vf')}
					/>
				</label>

				<!-- 15. Log soubor pro ověření -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Log soubor pro ověření</span>
						<Popover
							open={activePopover === 'verifylog'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'verifylog')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('verifylog')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<input
						class="input"
						name="verifylog"
						type="text"
						bind:value={formData.verifylog}
						on:focus={(e) => onInputFocus(e, 'verifylog')}
					/>
				</label>

				<!-- 16. Chování při chybách (conv) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Chování při chybách (conv)</span>
						<Popover
							open={activePopover === 'conv'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'conv')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('conv')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<input
						class="input"
						name="conv"
						type="text"
						bind:value={formData.conv}
						on:focus={(e) => onInputFocus(e, 'conv')}
					/>
				</label>

				<!-- 17. Log soubor pro chyby (errlog) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Log soubor pro chyby (errlog)</span>
						<Popover
							open={activePopover === 'errlog'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'errlog')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('errlog')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<input
						class="input"
						name="errlog"
						type="text"
						bind:value={formData.errlog}
						on:focus={(e) => onInputFocus(e, 'errlog')}
					/>
				</label>

				<!-- 18. Formát průběžného hashe (hashformat) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Formát průběžného hashe (hashformat)</span>
						<Popover
							open={activePopover === 'hashformat'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'hashformat')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('hashformat')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={hashformatOptions}
						defaultValue={[formData.hashformat]}
						value={[formData.hashformat]}
						onValueChange={(e) => (formData.hashformat = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 19. Formát celkového hashe (totalhashformat) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Formát celkového hashe (totalhashformat)</span>
						<Popover
							open={activePopover === 'totalhashformat'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'totalhashformat')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('totalhashformat')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={totalhashformatOptions}
						defaultValue={[formData.totalhashformat]}
						value={[formData.totalhashformat]}
						onValueChange={(e) => (formData.totalhashformat = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 20. Kdy provádět hashování (hashconv) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Kdy provádět hashování (hashconv)</span>
						<Popover
							open={activePopover === 'hashconv'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'hashconv')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('hashconv')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={hashconvOptions}
						defaultValue={[formData.hashconv]}
						value={[formData.hashconv]}
						onValueChange={(e) => (formData.hashconv = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 21. Porovnání bloků a zápis jen změněných (diffwr) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Porovnání bloků a zápis jen změněných (diffwr)</span>
						<Popover
							open={activePopover === 'diffwr'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'diffwr')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">
										{getExplanation('diffwr')}
									</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={diffwrOptions}
						defaultValue={[formData.diffwr]}
						value={[formData.diffwr]}
						onValueChange={(e) => (formData.diffwr = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 22. Velikost pro procentuální indikátor (sizeprobe) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Velikost pro procentuální indikátor (sizeprobe)</span>
						<Popover
							open={activePopover === 'sizeprobe'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'sizeprobe')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('sizeprobe')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={sizeprobeOptions}
						defaultValue={[formData.sizeprobe]}
						value={[formData.sizeprobe]}
						onValueChange={(e) => (formData.sizeprobe = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 23. Granularita chyb (read_retry_count) -->
				<label class="label">
					<span>Granularita chyb (read_retry_count)</span>
					<input
						class="input"
						name="granularity_sectors"
						type="number"
						bind:value={formData.granularity_sectors}
						on:focus={(e) => onInputFocus(e, 'granularity_sectors')}
						placeholder="2"
						required
					/>
				</label>

				<!-- 24. Poznámky (-N) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Poznámky (-N)</span>
						<Popover
							open={activePopover === 'notes'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'notes')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('notes')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={notesOptions}
						defaultValue={[formData.notes]}
						value={[formData.notes]}
						onValueChange={(e) => (formData.notes = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 25. Offset (-o) -->
				<label class="label">
					<span>Offset (-o)</span>
					<select
						class="select"
						name="offset"
						bind:value={formData.offset}
					>
						<option value="0">0 (výchozí)</option>
						<option value="ask">dotázat</option>
					</select>
				</label>

				<!-- 26. Velikost bufferu procesu (-p) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Velikost bufferu procesu (-p)</span>
						<Popover
							open={activePopover === 'process_buffer_size'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'process_buffer_size')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('process_buffer_size')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={processBufferSizeOptions}
						defaultValue={[formData.process_buffer_size]}
						value={[formData.process_buffer_size]}
						onValueChange={(e) => (formData.process_buffer_size = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 27. Bajtů na sektor (-P) -->
				<label class="label">
					<div class="flex items-center gap-2">
						<span>Bajtů na sektor (-P)</span>
						<Popover
							open={activePopover === 'bytes_per_sector'}
							onOpenChange={(e) => {
								if (!e.open) activePopover = null;
							}}
							positioning={{ placement: 'top' }}
							triggerBase="btn-icon preset-tonal"
							contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
							arrow
							arrowBackground="!bg-surface-200 dark:!bg-surface-800"
							zIndex="999"
						>
							{#snippet trigger()}
								<button on:click={() => (activePopover = 'bytes_per_sector')}>
									<Info />
								</button>
							{/snippet}
							{#snippet content()}
								<header class="flex justify-between">
									<p class="text-xl font-bold">Info</p>
									<button class="btn-icon" on:click={() => (activePopover = null)}>
										<X />
									</button>
								</header>
								<article>
									<p class="opacity-60">{getExplanation('bytes_per_sector')}</p>
								</article>
							{/snippet}
						</Popover>
					</div>
					<Combobox
						multiple={false}
						data={bytesPerSectorOptions}
						defaultValue={[formData.bytes_per_sector]}
						value={[formData.bytes_per_sector]}
						onValueChange={(e) => (formData.bytes_per_sector = e.value[0])}
						placeholder="Vyberte..."
					/>
				</label>

				<!-- 28. Tichý režim (-q) -->
				<label class="flex items-center">
					<input
						type="checkbox"
						class="checkbox"
						name="quiet_mode"
						bind:checked={formData.quiet_mode}
					/>
					<span class="ml-2">Tichý režim (-q)</span>
				</label>

				<!-- 29. Počet opakování při chybě čtení (-r) -->
				<label class="label">
					<span>Počet opakování při chybě čtení (-r)</span>
					<input
						class="input"
						name="read_retry_count"
						type="number"
						bind:value={formData.read_retry_count}
						on:focus={(e) => onInputFocus(e, 'read_retry_count')}
						placeholder="2"
					/>
				</label>

				<!-- 30. Přehodit bajtové páry (-s) -->
				<label class="flex items-center">
					<input
						type="checkbox"
						class="checkbox"
						name="swap_byte_pairs"
						bind:checked={formData.swap_byte_pairs}
					/>
					<span class="ml-2">Přehodit bajtové páry (-s)</span>
				</label>

				<!-- 31. Velikost segmentu souboru (-S) -->
				<label class="label">
					<span>Velikost segmentu souboru (-S)</span>
					<div class="input-group grid-cols-[auto_1fr]">
						<input
							class="ig-input"
							type="text"
							name="segment_size"
							bind:value={formData.segment_size}
							on:focus={(e) => onInputFocus(e, 'segment_size')}
							placeholder="1.4"
							required
						/>
						<select
							class="ig-select"
							bind:value={formData.segment_size_unit}
						>
							<option value="GiB">GiB</option>
							<option value="MiB">MiB</option>
							<option value="TiB">TiB</option>
						</select>
					</div>
				</label>

				<!-- 32. Podrobný výstup (-v) -->
				<label class="flex items-center">
					<input
						type="checkbox"
						class="checkbox"
						name="verbose_output"
						bind:checked={formData.verbose_output}
					/>
					<span class="ml-2">Podrobný výstup (-v)</span>
				</label>

				<!-- 33. Nulovat sektory při chybě čtení (-w) -->
				<label class="flex items-center">
					<input
						type="checkbox"
						class="checkbox"
						name="zero_on_read_error"
						bind:checked={formData.zero_on_read_error}
					/>
					<span class="ml-2">Nulovat sektory při chybě čtení (-w)</span>
				</label>

				<!-- 34. Použít chunk data (-x) -->
				<label class="flex items-center">
					<input
						type="checkbox"
						class="checkbox"
						name="use_chunk_data"
						bind:checked={formData.use_chunk_data}
					/>
					<span class="ml-2">Použít chunk data (-x)</span>
				</label>

				<div class="modal-footer">
					<button type="submit" class="btn variant-filled">
						{parent.buttonTextSubmit}
					</button>
				</div>
			</form>
		</div>
	{/snippet}
</Modal>

{#if showKeyboard}
	<!-- Vlastní komponenta pro virtuální klávesnici, která využívá SimpleKeyboard -->
	<VirtualKeyboard
		{activeInput}
		bind:formData
		on:closeKeyboard={() => {
			console.log('Keyboard closed');
			showKeyboard = false;
		}}
	/>
{/if}

<style>
	.modal-form {
		width: 90vh;
		height: 90vh;
		overflow-y: auto;
		overscroll-behavior: contain;
		-webkit-overflow-scrolling: touch;
		background-color: rgba(var(--color-surface-800) / 1);
		border-radius: 10px;
		padding: 20px;
	}
	.modal-form::-webkit-scrollbar {
		width: 1px;
		background-color: transparent;
	}
	.modal-header {
		margin-bottom: 1rem;
	}
	.label {
		display: flex;
		flex-direction: column;
		margin-bottom: 1rem;
	}
	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 1rem;
	}
	.input,
	.select,
	.ig-input,
	.ig-select {
		padding: 0.5rem;
		border: 1px solid #ccc;
		border-radius: 4px;
	}
	.input-group {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 0.5rem;
	}
	.flex {
		display: flex;
	}
	.items-center {
		align-items: center;
	}
	.ml-2 {
		margin-left: 0.5rem;
	}
	.gap-4 {
		gap: 1rem;
	}
</style>
