<script lang="ts">
	import { writable } from 'svelte/store';
	import { Combobox, Popover } from '@skeletonlabs/skeleton-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { X, Info } from 'lucide-svelte';
	import { goto } from '$app/navigation';
	import VirtualKeyboard from '$lib/components/VirtualKeyboard.svelte';

	// Stav popoverů – ponecháme původní zápis
	let confnamePopover = $state(false);
	let codepagePopover = $state(false);
	let sectorsPerReadPopover = $state(false);
	let bytesToReadPopover = $state(false);
	let compressionMethodPopover = $state(false);
	let compressionLevelPopover = $state(false);
	let hashTypesPopover = $state(false);
	let statusPopover = $state(false);
	let splitPopover = $state(false);
	let splitformatPopover = $state(false);
	let vfPopover = $state(false);
	let verifylogPopover = $state(false);
	let convPopover = $state(false);
	let errlogPopover = $state(false);
	let hashformatPopover = $state(false);
	let totalhashformatPopover = $state(false);
	let hashconvPopover = $state(false);
	let diffwrPopover = $state(false);
	let sizeprobePopover = $state(false);
	let processBufferSizePopover = $state(false);
	let bytesPerSectorPopover = $state(false);
	let notesPopover = $state(false);

	// Jednotlivé hodnoty formuláře jako reaktivní proměnné
	let confname = '';
	let codepage = 'ascii';
	let sectors_per_read = '64';
	let bytes_to_read = 'whole';
	let compression_method = 'deflate';
	let compression_level = 'none';
	let hash_types: any[] = $state([]);
	let ewf_format = 'encase6';
	let granularity_sectors = '2';
	let notes = 'ask';
	let offset = '0';
	let process_buffer_size = 'auto';
	let bytes_per_sector = 'auto';
	let read_retry_count = '2';
	let swap_byte_pairs = false;
	let segment_size_number = '1.4';
	let segment_size_unit = 'GiB';
	let segment_size = $derived(segment_size_number + segment_size_unit);
	let zero_on_read_error = false;
	let use_chunk_data = false;

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

	const compressionMethodOptions = [{ label: 'deflate', value: 'deflate' }];

	const compressionLevelOptions = [
		{ label: 'none (výchozí)', value: 'none' },
		{ label: 'empty-block', value: 'empty-block' },
		{ label: 'fast', value: 'fast' },
		{ label: 'best', value: 'best' }
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

	// Odeslání formuláře – sestavení objektu z jednotlivých hodnot
	async function onFormSubmit(): Promise<void> {
		const formData = {
			confname,
			codepage,
			sectors_per_read,
			bytes_to_read,
			compression_method,
			compression_level,
			hash_types,
			ewf_format,
			granularity_sectors,
			notes,
			offset,
			process_buffer_size,
			bytes_per_sector,
			read_retry_count,
			swap_byte_pairs,
			segment_size,
			segment_size_unit,
			zero_on_read_error,
			use_chunk_data
		};

		try {
			const result = await invoke('save_new_ewf_config', formData);
			console.log('Formulář odeslán a data uložena, výsledek:', result);
			goto('/dashboard/pre_configs/');
		} catch (error) {
			console.error('Chyba při odesílání formuláře:', error);
		}
	}

	function onInputFocus(event: Event, fieldName: string) {
		console.log('Input focused:', fieldName);
		// Logika focusu – již nepoužíváme virtuální klávesnici
	}

	let openState = $state(false);

	function popoverClose() {
		openState = false;
		confnamePopover = false;
		codepagePopover = false;
		sectorsPerReadPopover = false;
		bytesToReadPopover = false;
		compressionMethodPopover = false;
		compressionLevelPopover = false;
		hashTypesPopover = false;
		statusPopover = false;
		splitPopover = false;
		splitformatPopover = false;
		vfPopover = false;
		verifylogPopover = false;
		convPopover = false;
		errlogPopover = false;
		hashformatPopover = false;
		totalhashformatPopover = false;
		hashconvPopover = false;
		diffwrPopover = false;
		sizeprobePopover = false;
		processBufferSizePopover = false;
		bytesPerSectorPopover = false;
		notesPopover = false;
	}

	function getExplanation(field: string): string {
		switch (field) {
			case 'confname':
				return 'Název konfigurace slouží k identifikaci akvizice a je uveden v hlavičce výstupního souboru.';
			case 'codepage':
				return 'Určuje kódovou stránku pro interpretaci znaků v hlavičce. Možnosti: ascii (výchozí), windows-874, windows-932, až windows-1258.';
			case 'sectors_per_read':
				return 'Specifikuje počet sektorů, které se čtou najednou. Například 64 znamená, že se najednou přečte 64 sektorů.';
			case 'bytes_to_read':
				return 'Určuje počet bajtů k získání při každém čtecím cyklu. "celý disk" znamená kompletní akvizici, "dotázat" umožňuje interaktivní volbu.';
			case 'compression_method':
				return 'Vybere se kompresní metoda. Výchozí je "deflate".';
			case 'compression_level':
				return 'Nastavuje úroveň komprese. Možnosti: none, empty-block, fast, best.';
			case 'status':
				return 'Určuje, zda se mají zobrazovat status zprávy během akvizice.';
			case 'split':
				return 'Řídí rozdělení souboru na segmenty. "ask" umožňuje interaktivní volbu, "disabled" znamená, že se disk nerozdělí.';
			case 'hashformat':
				return 'Nastavuje formát průběžného hashe (např. hex nebo base64).';
			case 'totalhashformat':
				return 'Určuje formát celkového hashe (např. hex nebo base64).';
			case 'hashconv':
				return 'Definuje, zda se má hashování provádět před nebo po konverzi dat.';
			case 'diffwr':
				return 'Určuje, zda se mají zapisovat pouze změněné bloky (šetří místo).';
			case 'sizeprobe':
				return 'Nastavuje, zda se má velikostní indikátor počítat ze zdrojových nebo výstupních dat.';
			case 'process_buffer_size':
				return 'Definuje velikost bufferu pro zpracování dat; výchozí je automatická detekce.';
			case 'bytes_per_sector':
				return 'Určuje počet bajtů na sektor; "auto" znamená automatickou detekci, jinak lze zadat konkrétní hodnotu (např. 128, 256…).';
			case 'notes':
				return 'Určuje, zda se mají do výstupu zahrnout poznámky. "ask" vyvolá interaktivní volbu, "empty" znamená žádné poznámky.';
			case 'segment_size':
				return 'Určuje velikost segmentu souboru. Zadejte hodnotu a vyberte jednotku (GiB, MiB, TiB).';
			default:
				return 'Informace o tomto poli.';
		}
	}

	let showKeyboard = $state(false);
	
	let activeInput = '';
	let formData: Record<string, string> = {};

	function openKeyboard(inputName: string) {
		console.log('openKeyboard called for:', inputName);
		activeInput = inputName;
		showKeyboard = true;
	}

	function closeKeyboard() {
		showKeyboard = false;
		activeInput = '';
	}
</script>

<main class="page-container">
	<form
		onsubmit={(e) => {
			e.preventDefault();
			onFormSubmit();
		}}
		class="modal-form"
	>
		<!-- 1. Název konfigurace -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Název konfigurace</span>
				<Popover
					open={openState}
					onOpenChange={(e) => (openState = e.open)}
					positioning={{ placement: 'top', offset: { mainAxis: 8, crossAxis: 0 } }}
					modal={true}
					closeOnInteractOutside={false}
					closeOnEscape={false}
					zIndex="100"
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
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
				bind:value={confname}
				onfocus={() => openKeyboard('confname')}
				required
			/>
		</label>

		<!-- 2. Kódová stránka -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Kódová stránka</span>
				<Popover
					open={codepagePopover}
					onOpenChange={(e) => (codepagePopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
						</header>
						<article>
							<p class="opacity-60">{getExplanation('codepage')}</p>
						</article>
					{/snippet}
				</Popover>
			</div>
			<select class="select" name="codepage" bind:value={codepage}>
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
					open={sectorsPerReadPopover}
					onOpenChange={(e) => (sectorsPerReadPopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
						</header>
						<article>
							<p class="opacity-60">{getExplanation('sectors_per_read')}</p>
						</article>
					{/snippet}
				</Popover>
			</div>
			<Combobox
				multiple={false}
				data={sectorsPerReadOptions}
				defaultValue={[sectors_per_read]}
				value={[sectors_per_read]}
				onValueChange={(e) => (sectors_per_read = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- 4. Počet bajtů k získání (-B) -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Počet bajtů k získání (-B)</span>
				<Popover
					open={bytesToReadPopover}
					onOpenChange={(e) => (bytesToReadPopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
						</header>
						<article>
							<p class="opacity-60">{getExplanation('bytes_to_read')}</p>
						</article>
					{/snippet}
				</Popover>
			</div>
			<Combobox
				multiple={false}
				data={bytesToReadOptions}
				defaultValue={[bytes_to_read]}
				value={[bytes_to_read]}
				onValueChange={(e) => (bytes_to_read = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- 5. Metoda komprese (-c) -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Metoda komprese (-c)</span>
				<Popover
					open={compressionMethodPopover}
					onOpenChange={(e) => (compressionMethodPopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
						</header>
						<article>
							<p class="opacity-60">{getExplanation('compression_method')}</p>
						</article>
					{/snippet}
				</Popover>
			</div>
			<Combobox
				multiple={false}
				data={compressionMethodOptions}
				defaultValue={[compression_method]}
				value={[compression_method]}
				onValueChange={(e) => (compression_method = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- 6. Úroveň komprese (-c) -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Úroveň komprese (-c)</span>
				<Popover
					open={compressionLevelPopover}
					onOpenChange={(e) => (compressionLevelPopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
						</header>
						<article>
							<p class="opacity-60">{getExplanation('compression_level')}</p>
						</article>
					{/snippet}
				</Popover>
			</div>
			<Combobox
				multiple={false}
				data={compressionLevelOptions}
				defaultValue={[compression_level]}
				value={[compression_level]}
				onValueChange={(e) => (compression_level = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- 7. Hashovací algoritmy (-d) – MD5 vždy zahrnuto -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Hashovací algoritmy (-d) – MD5 vždy zahrnuto</span>
				<Popover
					open={hashTypesPopover}
					onOpenChange={(e) => (hashTypesPopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
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
					<input type="checkbox" class="checkbox" bind:group={hash_types} value="sha1" />
					<span class="ml-2">SHA1</span>
				</label>
				<label class="flex items-center">
					<input type="checkbox" class="checkbox" bind:group={hash_types} value="sha256" />
					<span class="ml-2">SHA256</span>
				</label>
			</div>
		</label>

		<!-- 23. Granularita chyb (read_retry_count) -->
		<label class="label">
			<span>Granularita chyb (read_retry_count)</span>
			<input
				class="input"
				name="granularity_sectors"
				type="number"
				bind:value={granularity_sectors}
				onfocus={() => openKeyboard('granularity_sectors')}
				placeholder="2"
				required
			/>
		</label>

		<!-- 24. Poznámky (-N) -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Poznámky (-N)</span>
				<Popover
					open={notesPopover}
					onOpenChange={(e) => (notesPopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
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
				defaultValue={[notes]}
				value={[notes]}
				onValueChange={(e) => (notes = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- 25. Offset (-o) -->
		<label class="label">
			<span>Offset (-o)</span>
			<select class="select" name="offset" bind:value={offset}>
				<option value="0">0 (výchozí)</option>
				<option value="ask">dotázat</option>
			</select>
		</label>

		<!-- 26. Velikost bufferu procesu (-p) -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Velikost bufferu procesu (-p)</span>
				<Popover
					open={processBufferSizePopover}
					onOpenChange={(e) => (processBufferSizePopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
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
				defaultValue={[process_buffer_size]}
				value={[process_buffer_size]}
				onValueChange={(e) => (process_buffer_size = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- 27. Bajtů na sektor (-P) -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Bajtů na sektor (-P)</span>
				<Popover
					open={bytesPerSectorPopover}
					onOpenChange={(e) => (bytesPerSectorPopover = e.open)}
					positioning={{ placement: 'top' }}
					triggerBase="btn-icon preset-tonal"
					contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
					arrow
					arrowBackground="!bg-surface-200 dark:!bg-surface-800"
					zIndex="999"
				>
					{#snippet trigger()}
						<Info />
					{/snippet}
					{#snippet content()}
						<header class="flex justify-between">
							<p class="text-xl font-bold">Info</p>
							<button type="button" class="btn-icon" onclick={popoverClose}><X /></button>
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
				defaultValue={[bytes_per_sector]}
				value={[bytes_per_sector]}
				onValueChange={(e) => (bytes_per_sector = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- 29. Počet opakování při chybě čtení (-r) -->
		<label class="label">
			<span>Počet opakování při chybě čtení (-r)</span>
			<input
				class="input"
				name="read_retry_count"
				type="number"
				bind:value={read_retry_count}
				onfocus={() => openKeyboard('read_retry_count')}
				placeholder="2"
			/>
		</label>

		<!-- 30. Přehodit bajtové páry (-s) -->
		<label class="flex items-center">
			<input
				type="checkbox"
				class="checkbox"
				name="swap_byte_pairs"
				bind:checked={swap_byte_pairs}
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
					bind:value={segment_size_number}
					onfocus={() => openKeyboard('segment_size')}
					placeholder="1.4"
					required
				/>
				<select class="ig-select" bind:value={segment_size_unit}>
					<option value="GiB">GiB</option>
					<option value="MiB">MiB</option>
					<option value="TiB">TiB</option>
				</select>
			</div>
		</label>

		<!-- 33. Nulovat sektory při chybě čtení (-w) -->
		<label class="flex items-center">
			<input
				type="checkbox"
				class="checkbox"
				name="zero_on_read_error"
				bind:checked={zero_on_read_error}
			/>
			<span class="ml-2">Nulovat sektory při chybě čtení (-w)</span>
		</label>

		<!-- 34. Použít chunk data (-x) -->
		<label class="flex items-center">
			<input type="checkbox" class="checkbox" name="use_chunk_data" bind:checked={use_chunk_data} />
			<span class="ml-2">Použít chunk data (-x)</span>
		</label>

		<div class="flex justify-end">
			<button type="submit" class="btn preset-filled-primary-500">Uložit</button>
		</div>
	</form>
</main>

<VirtualKeyboard
	bind:showKeyboard
	bind:activeInput
	bind:formData
	on:closeKeyboard={closeKeyboard}
/>

<style lang="postcss">
	.page-container {
		width: 100%;
		padding: 20px;
	}
	.modal-form {
		width: 90vh;
		max-width: 800px;
		margin: auto;
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
	.label {
		display: flex;
		flex-direction: column;
		margin-bottom: 1rem;
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
