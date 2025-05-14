<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import VirtualKeyboard from '$lib/components/VirtualKeyboard.svelte';
	import { Popover, Combobox } from '@skeletonlabs/skeleton-svelte';
	import { Info, X } from 'lucide-svelte';

	interface NewEWFConfig {
		[key: string]: string | boolean | string[];
		confname: string;
		codepage: string;
		sectors_per_read: string;
		bytes_to_read: string;
		compression_method: string;
		compression_level: string;
		hash_types: string[];
		ewf_format: string;
		granularity_sectors: string;
		notes: string;
		offset: string;
		process_buffer_size: string;
		bytes_per_sector: string;
		read_retry_count: string;
		swap_byte_pairs: boolean;
		segment_size: string;
		zero_on_read_error: boolean;
		use_chunk_data: boolean;
	}

	let formData: NewEWFConfig = {
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
		read_retry_count: '2',
		swap_byte_pairs: false,
		segment_size: '1.4GiB',
		zero_on_read_error: false,
		use_chunk_data: false
	};

	let confnamePopover = false;
	let ewfFormatPopover = false;
	let codepagePopover = false;
	let sectorsPerReadPopover = false;
	let bytesToReadPopover = false;
	let compressionMethodPopover = false;
	let compressionLevelPopover = false;
	let hashTypesPopover = false;
	let granularityPopover = false;
	let notesPopover = false;
	let offsetPopover = false;
	let processBufferSizePopover = false;
	let bytesPerSectorPopover = false;
	let readRetryCountPopover = false;
	let swapBytePairsPopover = false;
	let segmentSizePopover = false;
	let zeroOnReadErrorPopover = false;
	let useChunkDataPopover = false;

	const codepageOptions = [
		{ label: 'ascii (výchozí)', value: 'ascii' },
		{ label: 'windows-874', value: 'windows-874' },
		{ label: 'windows-932', value: 'windows-932' },
		{ label: 'windows-936', value: 'windows-936' },
		{ label: 'windows-949', value: 'windows-949' },
		{ label: 'windows-950', value: 'windows-950' },
		{ label: 'windows-1250', value: 'windows-1250' },
		{ label: 'windows-1251', value: 'windows-1251' },
		{ label: 'windows-1252', value: 'windows-1252' },
		{ label: 'windows-1253', value: 'windows-1253' },
		{ label: 'windows-1254', value: 'windows-1254' },
		{ label: 'windows-1255', value: 'windows-1255' },
		{ label: 'windows-1256', value: 'windows-1256' },
		{ label: 'windows-1257', value: 'windows-1257' },
		{ label: 'windows-1258', value: 'windows-1258' }
	];

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
		{ label: '128 sektorů', value: '128' },
		{ label: '256 sektorů', value: '256' },
		{ label: '512 sektorů', value: '512' },
		{ label: '1024 sektorů', value: '1024' },
		{ label: '2048 sektorů', value: '2048' },
		{ label: '4096 sektorů', value: '4096' },
		{ label: '8192 sektorů', value: '8192' },
		{ label: '16384 sektorů', value: '16384' },
		{ label: '262144 sektorů', value: '262144' }
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

	const ewfFormatOptions = [
		{ label: 'EWF', value: 'ewf' },
		{ label: 'SMART', value: 'smart' },
		{ label: 'FTK', value: 'ftk' },
		{ label: 'EnCase 1', value: 'encase1' },
		{ label: 'EnCase 2', value: 'encase2' },
		{ label: 'EnCase 3', value: 'encase3' },
		{ label: 'EnCase 4', value: 'encase4' },
		{ label: 'EnCase 5', value: 'encase5' },
		{ label: 'EnCase 6', value: 'encase6' },
		{ label: 'LiNE n5', value: 'linen5' },
		{ label: 'LiNE n6', value: 'linen6' },
		{ label: 'EWFx', value: 'ewfx' }
	];

	const notesOptions = [
		{ label: 'dotázat (výchozí)', value: 'ask' },
		{ label: 'neuvádět', value: 'empty' }
	];

	function getExplanation(field: string): string {
		switch (field) {
			case 'confname':
				return 'Název této konfigurace. Slouží pouze pro tvou orientaci a identifikaci akvizice v systému.';
			case 'codepage':
				return 'Kódová stránka pro textová pole v metadatech. Pokud nevíš, ponech "ascii". Pro speciální znaky použij odpovídající Windows kódovou stránku.';
			case 'sectors_per_read':
				return 'Počet sektorů, které se čtou najednou při akvizici (-b). Vyšší hodnota může zvýšit rychlost čtení, ale maximální povolená hodnota je 32768. Nastav podle výkonu a stability zařízení.';
			case 'bytes_to_read':
				return 'Určuje, zda se bude číst celý disk ("whole") nebo se program zeptá na rozsah ("ask"). Lepší ponechat "celý disk".';
			case 'compression_method':
				return 'Metoda komprese dat v obrazu. Výchozí je "deflate".';
			case 'compression_level':
				return 'Úroveň komprese: "none" (bez komprese, nejrychlejší), "empty-block" (komprimuje jen prázdné bloky), "fast" (rychlá komprese), "best" (nejvyšší komprese, ale pomalejší).';
			case 'hash_types':
				return 'Dodatečné hashovací algoritmy pro ověření integrity dat. MD5 je vždy zahrnuto. Přidej SHA1 nebo SHA256 pro vyšší bezpečnost.';
			case 'ewf_format':
				return 'Formát výsledného EWF obrazu. Výchozí "encase6" je nejnovější a nejkompatibilnější. Změň jen pokud máš speciální požadavky na kompatibilitu.';
			case 'granularity_sectors':
				return 'Granularita v sektorech (velikost nejmenší jednotky pro zápis do obrazu). Výchozí hodnota je 2. Obvykle není potřeba měnit.';
			case 'notes':
				return 'Poznámky k akvizici, které budou uloženy v metadatech obrazu. "ask" znamená, že se program zeptá při spuštění.';
			case 'offset':
				return 'Offset v bajtech od začátku disku, odkud začít číst. Výchozí je 0 (čte se od začátku). Změň pouze pro speciální případy.';
			case 'process_buffer_size':
				return 'Velikost bufferu procesu (-p). Ovlivňuje rychlost akvizice – čím větší, tím rychlejší (pokud je dost RAM). Doporučujeme nastavit na několik GB, ale nikdy ne více, než kolik je volné RAM. Pro systém s 32 GB RAM lze použít až 12 GB, pokud běží dva procesy současně.';
			case 'bytes_per_sector':
				return 'Počet bajtů na sektor (-P). Toto je fyzická vlastnost disku (typicky 512 nebo 4096). Obvykle ponech "auto" – program detekuje správnou hodnotu sám. Neměň, pokud nevíš přesně proč.';
			case 'read_retry_count':
				return 'Počet opakování čtení při chybě (-r). Vyšší hodnota znamená, že se program vícekrát pokusí přečíst poškozený sektor, což může zpomalit akvizici, ale zvýšit šanci na úspěšné přečtení.';
			case 'swap_byte_pairs':
				return 'Přehazování bajtových párů při čtení (-s). Používá se pouze pro speciální typy zařízení s nestandardním pořadím bajtů. Většinou ponech vypnuté.';
			case 'segment_size':
				return 'Velikost segmentu souboru (-S). Určuje maximální velikost jednoho souboru obrazu. Pokud je obraz větší, rozdělí se na více souborů. Nastav podle toho, jak velké soubory potřebuješ (např. kvůli limitům filesystému nebo pro snadnější přenos). Pro běžné použití nastav několik GB až stovky GB.';
			case 'zero_on_read_error':
				return 'Pokud je zapnuto, sektory, které nelze přečíst, budou v obrazu nahrazeny nulami (-w). Jinak zůstanou nečitelné sektory nezměněné.';
			case 'use_chunk_data':
				return 'Použít chunk data při čtení (-x). Speciální režim pro některé typy zařízení nebo pokročilé použití. Většinou ponech vypnuté.';
			default:
				return 'Informace o tomto poli.';
		}
	}

	function handleKeyboardInput(field: string, value: string) {
		formData[field] = value;
	}

	let showKeyboard = false;
	let activeInput = '';

	function openKeyboard(inputName: string) {
		activeInput = inputName;
		showKeyboard = true;
	}

	function closeKeyboard() {
		showKeyboard = false;
		activeInput = '';
	}

	function popoverClose() {
		confnamePopover = false;
		ewfFormatPopover = false;
		codepagePopover = false;
		sectorsPerReadPopover = false;
		bytesToReadPopover = false;
		compressionMethodPopover = false;
		compressionLevelPopover = false;
		hashTypesPopover = false;
		granularityPopover = false;
		notesPopover = false;
		offsetPopover = false;
		processBufferSizePopover = false;
		bytesPerSectorPopover = false;
		readRetryCountPopover = false;
		swapBytePairsPopover = false;
		segmentSizePopover = false;
		zeroOnReadErrorPopover = false;
		useChunkDataPopover = false;
	}

	async function onFormSubmit(): Promise<void> {
		try {
			await invoke('save_new_ewf_config', { ...formData });
			goto('/dashboard/pre_configs');
		} catch (error) {
			console.error('Chyba při odesílání formuláře:', error);
		}
	}
</script>

<main class="page-wrapper">
	<form class="modal-form" on:submit|preventDefault={onFormSubmit}>
		<!-- Název konfigurace -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Název konfigurace</span>
				<Popover
					open={confnamePopover}
					onOpenChange={(e) => (confnamePopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (confnamePopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('confname')}
					{/snippet}
				</Popover>
			</div>
			<input
				class="input"
				name="confname"
				type="text"
				maxlength="12"
				bind:value={formData.confname}
				on:focus={() => openKeyboard('confname')}
				required
			/>
		</label>

		<!-- Formát EWF -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Výstupní formát (-f)</span>
				<Popover
					open={ewfFormatPopover}
					onOpenChange={(e) => (ewfFormatPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (ewfFormatPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('ewf_format')}
					{/snippet}
				</Popover>
			</div>
			<Combobox
				multiple={false}
				data={ewfFormatOptions}
				defaultValue={[formData.ewf_format]}
				value={[formData.ewf_format]}
				onValueChange={(e) => (formData.ewf_format = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- Kódová stránka -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Kódová stránka</span>
				<Popover
					open={codepagePopover}
					onOpenChange={(e) => (codepagePopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (codepagePopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('codepage')}
					{/snippet}
				</Popover>
			</div>
			<Combobox
				multiple={false}
				data={codepageOptions}
				defaultValue={[formData.codepage]}
				value={[formData.codepage]}
				onValueChange={(e) => (formData.codepage = e.value[0])}
				placeholder="Vyberte..."
			/>
		</label>

		<!-- Počet sektorů na čtení -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Počet sektorů na čtení (-b)</span>
				<Popover
					open={sectorsPerReadPopover}
					onOpenChange={(e) => (sectorsPerReadPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (sectorsPerReadPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('sectors_per_read')}
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

		<!-- Počet bajtů k získání -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Počet bajtů k získání (-B)</span>
				<Popover
					open={bytesToReadPopover}
					onOpenChange={(e) => (bytesToReadPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (bytesToReadPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('bytes_to_read')}
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

		<!-- Metoda komprese -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Metoda komprese (-c)</span>
				<Popover
					open={compressionMethodPopover}
					onOpenChange={(e) => (compressionMethodPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (compressionMethodPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('compression_method')}
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

		<!-- Úroveň komprese -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Úroveň komprese (-c)</span>
				<Popover
					open={compressionLevelPopover}
					onOpenChange={(e) => (compressionLevelPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (compressionLevelPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('compression_level')}
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

		<!-- Hashovací algoritmy – MD5 vždy zahrnuto -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Hashovací algoritmy – MD5 vždy zahrnuto</span>
				<Popover
					open={hashTypesPopover}
					onOpenChange={(e) => (hashTypesPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (hashTypesPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('hash_types')}
					{/snippet}
				</Popover>
			</div>
			<div class="flex gap-4">
				<label class="flex items-center">
					<input type="checkbox" class="checkbox" bind:group={formData.hash_types} value="sha1" />
					<span class="ml-2">SHA1</span>
				</label>
				<label class="flex items-center">
					<input type="checkbox" class="checkbox" bind:group={formData.hash_types} value="sha256" />
					<span class="ml-2">SHA256</span>
				</label>
			</div>
		</label>

		<!-- Granularita sektorů -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Granularita sektorů</span>
				<Popover
					open={granularityPopover}
					onOpenChange={(e) => (granularityPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (granularityPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('granularity_sectors')}
					{/snippet}
				</Popover>
			</div>
			<input
				class="input"
				name="granularity_sectors"
				type="text"
				bind:value={formData.granularity_sectors}
				on:focus={() => openKeyboard('granularity_sectors')}
				placeholder="2"
				required
			/>
		</label>

		<!-- Poznámky -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Poznámky (-N)</span>
				<Popover
					open={notesPopover}
					onOpenChange={(e) => (notesPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (notesPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('notes')}
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

		<!-- Offset -->
		<label class="label">
			<span>Offset (-o)</span>
			<select class="select" name="offset" bind:value={formData.offset}>
				<option value="0">0 (výchozí)</option>
				<option value="ask">dotázat</option>
			</select>
		</label>

		<!-- Velikost bufferu procesu -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Velikost bufferu procesu (-p)</span>
				<Popover
					open={processBufferSizePopover}
					onOpenChange={(e) => (processBufferSizePopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (processBufferSizePopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('process_buffer_size')}
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

		<!-- Bajtů na sektor -->
		<label class="label">
			<div class="flex items-center gap-2">
				<span>Bajtů na sektor (-P)</span>
				<Popover
					open={bytesPerSectorPopover}
					onOpenChange={(e) => (bytesPerSectorPopover = e.open)}
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
						<div class="mb-2 flex items-center justify-between">
							<h2 class="text-lg font-bold">Info</h2>
							<button class="btn-icon" on:click={() => (bytesPerSectorPopover = false)}>
								<X />
							</button>
						</div>
						{getExplanation('bytes_per_sector')}
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

		<!-- Počet opakování při chybě čtení -->
		<label class="label">
			<span>Počet opakování při chybě čtení (-r)</span>
			<input
				class="input"
				name="read_retry_count"
				type="text"
				bind:value={formData.read_retry_count}
				on:focus={() => openKeyboard('read_retry_count')}
				placeholder="2"
			/>
		</label>

		<!-- Přehodit bajtové páry -->
		<label class="flex items-center">
			<input
				type="checkbox"
				class="checkbox"
				name="swap_byte_pairs"
				bind:checked={formData.swap_byte_pairs}
			/>
			<span class="ml-2">Přehodit bajtové páry (-s)</span>
		</label>

		<!-- Velikost segmentu souboru -->
		<label class="label">
			<span>Velikost segmentu souboru (-S)</span>
			<input
				class="input"
				type="text"
				name="segment_size"
				bind:value={formData.segment_size}
				on:focus={() => openKeyboard('segment_size')}
				placeholder="1.4GiB"
				required
			/>
		</label>

		<!-- Nulovat sektory při chybě čtení -->
		<label class="flex items-center">
			<input
				type="checkbox"
				class="checkbox"
				name="zero_on_read_error"
				bind:checked={formData.zero_on_read_error}
			/>
			<span class="ml-2">Nulovat sektory při chybě čtení (-w)</span>
		</label>

		<!-- Použít chunk data -->
		<label class="flex items-center">
			<input
				type="checkbox"
				class="checkbox"
				name="use_chunk_data"
				bind:checked={formData.use_chunk_data}
			/>
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
	onInputChange={handleKeyboardInput}
/>

<style>
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
	.page-wrapper {
		padding: 2rem;
		max-width: 900px;
		margin: auto;
	}
	.label {
		display: flex;
		flex-direction: column;
		margin-bottom: 1rem;
	}
	.input,
	.select,
	.ig-input {
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
