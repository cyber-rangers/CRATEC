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

    function getExplanation(field: string): string {
        switch (field) {
            case 'confname':
                return 'Název konfigurace pro identifikaci akvizice.';
            case 'codepage':
                return 'Kódová stránka (např. ascii, windows-1250, apod.).';
            case 'sectors_per_read':
                return 'Počet sektorů, které se čtou najednou.';
            case 'bytes_to_read':
                return 'Určuje, zda se čte celý disk nebo se dotazuje.';
            case 'compression_method':
                return 'Metoda komprese, výchozí je deflate.';
            case 'compression_level':
                return 'Úroveň komprese: none, empty-block, fast, best.';
            case 'hash_types':
                return 'Dodatečné hashovací algoritmy, kromě výchozího MD5.';
            case 'ewf_format':
                return 'Formát pro EWF akvizici, výchozí je encase6.';
            case 'granularity_sectors':
                return 'Granularita v sektorech, výchozí hodnota je 2.';
            case 'notes':
                return 'Poznámky k akvizici, výchozí je "ask".';
            case 'offset':
                return 'Offset při čtení, výchozí je 0.';
            case 'process_buffer_size':
                return 'Velikost bufferu pro zpracování dat, výchozí je auto.';
            case 'bytes_per_sector':
                return 'Počet bajtů na sektor, výchozí je auto.';
            case 'read_retry_count':
                return 'Počet opakování při chybě čtení.';
            case 'swap_byte_pairs':
                return 'Přehazování bajtových párů při čtení.';
            case 'segment_size':
                return 'Velikost segmentu souboru včetně jednotky (např. 1.4GiB).';
            case 'zero_on_read_error':
                return 'Nulovat sektory při chybách čtení.';
            case 'use_chunk_data':
                return 'Použít chunk data při čtení.';
            default:
                return 'Informace o tomto poli.';
        }
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
                        <div class="flex justify-between items-center mb-2">
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
                        <div class="flex justify-between items-center mb-2">
                            <h2 class="text-lg font-bold">Info</h2>
                            <button class="btn-icon" on:click={() => (codepagePopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('codepage')}
                    {/snippet}
                </Popover>
            </div>
            <select class="select" name="codepage" bind:value={formData.codepage}>
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
                        <div class="flex justify-between items-center mb-2">
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
                        <div class="flex justify-between items-center mb-2">
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
                        <div class="flex justify-between items-center mb-2">
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
                        <div class="flex justify-between items-center mb-2">
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
                        <div class="flex justify-between items-center mb-2">
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
            <span>Granularita sektorů (granularity_sectors)</span>
            <input
                class="input"
                name="granularity_sectors"
                type="number"
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
                        <div class="flex justify-between items-center mb-2">
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
                        <div class="flex justify-between items-center mb-2">
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
                        <div class="flex justify-between items-center mb-2">
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
                type="number"
                bind:value={formData.read_retry_count}
                on:focus={() => openKeyboard('read_retry_count')}
                placeholder="2"
            />
        </label>

        <!-- Přehodit bajtové páry -->
        <label class="flex items-center">
            <input type="checkbox" class="checkbox" name="swap_byte_pairs" bind:checked={formData.swap_byte_pairs} />
            <span class="ml-2">Přehodit bajtové páry (-s)</span>
        </label>

        <!-- Velikost segmentu souboru -->
        <label class="label">
            <span>Velikost segmentu souboru (-S)</span>
            <div class="input-group">
                <input
                    class="ig-input"
                    type="text"
                    name="segment_size"
                    bind:value={formData.segment_size}
                    on:focus={() => openKeyboard('segment_size')}
                    placeholder="1.4GiB"
                    required
                />
            </div>
        </label>

        <!-- Nulovat sektory při chybě čtení -->
        <label class="flex items-center">
            <input type="checkbox" class="checkbox" name="zero_on_read_error" bind:checked={formData.zero_on_read_error} />
            <span class="ml-2">Nulovat sektory při chybě čtení (-w)</span>
        </label>

        <!-- Použít chunk data -->
        <label class="flex items-center">
            <input type="checkbox" class="checkbox" name="use_chunk_data" bind:checked={formData.use_chunk_data} />
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
