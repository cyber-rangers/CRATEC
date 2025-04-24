<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { goto } from '$app/navigation';
    import VirtualKeyboard from '$lib/components/VirtualKeyboard.svelte';
    import { Popover, Combobox } from '@skeletonlabs/skeleton-svelte';
    import { Info, X } from 'lucide-svelte';

    interface NewDDConfig {
        [key: string]: string | string[];
        confname: string;
        format: string;
        limit_mode: string;
        offset: string; // místo seek + skip
        hash_types: string[];
        hashwindow_value: string; 
        hashwindow_unit: string;  
        split_value: string;      
        split_unit: string;       
        vf: string;
        diffwr: string;
        notes: string;
    }

    let formData: NewDDConfig = {
        confname: '',
        format: 'auto',
        limit_mode: 'whole',
        offset: '0',
        hash_types: ['md5'],
        hashwindow_value: '5',
        hashwindow_unit: 'GB',
        split_value: '',
        split_unit: 'MB',
        vf: 'off',
        diffwr: 'off',
        notes: 'ask'
    };

    let confnamePopover = false;
    let formatPopover = false;
    let limitPopover = false;
    let seekPopover = false;
    let hashPopover = false;
    let hashWindowPopover = false;
    let splitPopover = false;
    let vfPopover = false;
    let diffwrPopover = false;
    let notesPopover = false;

    const formatOptions = [
        { label: 'Automatická detekce', value: 'auto' },
        { label: '512', value: '512' },
        { label: '1024', value: '1024' },
        { label: '2048', value: '2048' }
    ];

    const limitOptions = [
        { label: 'Celý disk', value: 'whole' },
        { label: 'Dotázat', value: 'ask' }
    ];

    const offsetOptions = [
        { label: '0', value: '0' },
        { label: 'Dotázat', value: 'ask' }
    ];

    const diffwrOptions = [
        { label: 'off', value: 'off' },
        { label: 'on', value: 'on' }
    ];

    const notesOptions = [
        { label: 'Dotázat se', value: 'ask' },
        { label: 'Neuvádět', value: 'none' }
    ];

    const hashTypeOptions = [
        { label: 'MD5', value: 'md5' },
        { label: 'SHA1', value: 'sha1' },
        { label: 'SHA256', value: 'sha256' },
        { label: 'SHA384', value: 'sha384' },
        { label: 'SHA512', value: 'sha512' }
    ];

    const hashUnitOptions = [
        { label: 'MB', value: 'MB' },
        { label: 'GB', value: 'GB' }
    ];

    const splitUnitOptions = [
        { label: 'MB', value: 'MB' },
        { label: 'GB', value: 'GB' }
    ];

    function getExplanation(field: string): string {
        switch (field) {
            case 'confname':
                return 'Název konfigurace pro identifikaci akvizice.';
            case 'format':
                return 'Velikost bloku (auto, 512, 1024 nebo 2048).';
            case 'limit_mode':
                return 'Určuje, zda číst celý disk nebo zobrazit dotaz.';
            case 'offset':
                return 'Vstupní i výstupní offset (nahrazuje původní seek/skip).';
            case 'hash_types':
                return 'Seznam hashovacích algoritmů (lze vybrat více).';
            case 'hash_window':
                return 'Velikost mezibloku pro průběžný hash.';
            case 'split':
                return 'Rozdělení výstupního souboru na více částí. "whole" znamená nerozdělovat.';
            case 'vf':
                return 'Dodatečné ověření, které prodlouží proces a nelze jej použít, pokud je výstup rozdělen na více souborů.';
            case 'diffwr':
                return 'Porovnání/změna bloků. off = vypnuto, on = zapnuto.';
            case 'notes':
                return 'Poznámky k akvizici: Dotázat nebo Neuvádět.';
            default:
                return 'Popis není k dispozici.';
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
        formatPopover = false;
        limitPopover = false;
        seekPopover = false;
        hashPopover = false;
        hashWindowPopover = false;
        splitPopover = false;
        vfPopover = false;
        diffwrPopover = false;
        notesPopover = false;
    }

    function toggleHashType(value: string) {
        if (formData.hash_types.includes(value)) {
            formData.hash_types = formData.hash_types.filter(v => v !== value);
        } else {
            formData.hash_types = [...formData.hash_types, value];
        }
    }

    function toggleVF() {
        formData.vf = formData.vf === 'on' ? 'off' : 'on';
    }

    function isWholeSplit() {
        return !formData.split_value.trim();
    }

    async function onFormSubmit(): Promise<void> {
        try {
            if (!formData.hashwindow_value.trim()) {
                formData.hashwindow_value = 'whole';
            }
            if (!formData.split_value.trim()) {
                formData.split_value = 'whole';
            }

            // Převod jednotek pro ukládání do DB
            const unitMap = { GB: 'G', MB: 'M' };

            // Přepiš split_unit a hashwindow_unit na správný formát
            const split_unit = unitMap[formData.split_unit as 'GB' | 'MB'];
            const hashwindow_unit = unitMap[formData.hashwindow_unit as 'GB' | 'MB'];

            await invoke('save_new_dd_config', {
                confname: formData.confname,
                format: formData.format,
                limit_mode: formData.limit_mode,
                offset: formData.offset,
                hash_types: formData.hash_types,
                hashwindow_value: formData.hashwindow_value,
                hashwindow_unit, // zde už bude 'G' nebo 'M'
                split_value: formData.split_value,
                split_unit,      // zde už bude 'G' nebo 'M'
                vf: formData.vf,
                diffwr: formData.diffwr,
                notes: formData.notes,
            });

            goto('/dashboard/pre_configs');
        } catch (error) {
            console.error('Chyba při ukládání:', error);
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
                required
                on:focus={() => openKeyboard('confname')}
                readonly
            />
        </label>

        <!-- Formát bloků -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Formát bloků</span>
                <Popover
                    open={formatPopover}
                    onOpenChange={(e) => (formatPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (formatPopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('format')}
                    {/snippet}
                </Popover>
            </div>
            <Combobox
                multiple={false}
                data={formatOptions}
                defaultValue={[formData.format]}
                value={[formData.format]}
                onValueChange={(e) => (formData.format = e.value[0])}
                placeholder="Vyberte..."
            />
        </label>

        <!-- Limit -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Limit (celý disk/dotázat)</span>
                <Popover
                    open={limitPopover}
                    onOpenChange={(e) => (limitPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (limitPopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('limit_mode')}
                    {/snippet}
                </Popover>
            </div>
            <Combobox
                multiple={false}
                data={limitOptions}
                defaultValue={[formData.limit_mode]}
                value={[formData.limit_mode]}
                onValueChange={(e) => (formData.limit_mode = e.value[0])}
                placeholder="Vyberte..."
            />
        </label>

        <!-- Offset (sjednocené) -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Offset</span>
                <Popover
                    open={seekPopover}
                    onOpenChange={(e) => (seekPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (seekPopover = false)}>
                                <X />
                            </button>
                        </div>
                        Vstupní i výstupní offset (nahrazuje původní seek/skip).
                    {/snippet}
                </Popover>
            </div>
            <Combobox
                multiple={false}
                data={offsetOptions}
                defaultValue={[formData.offset]}
                value={[formData.offset]}
                onValueChange={(e) => (formData.offset = e.value[0])}
                placeholder="Vyberte..."
            />
        </label>

        <!-- Hashovací algoritmy -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Vybrané hashovací algoritmy</span>
                <Popover
                    open={hashPopover}
                    onOpenChange={(e) => (hashPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (hashPopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('hash_types')}
                    {/snippet}
                </Popover>
            </div>
            <div class="flex gap-4">
                {#each hashTypeOptions as option}
                    <label class="flex items-center">
                        <input
                            type="checkbox"
                            class="checkbox"
                            value={option.value}
                            checked={formData.hash_types.includes(option.value)}
                            on:change={() => toggleHashType(option.value)}
                        />
                        <span class="ml-2">{option.label}</span>
                    </label>
                {/each}
            </div>
        </label>

        <!-- Hash okno -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Hash okno</span>
                <Popover
                    open={hashWindowPopover}
                    onOpenChange={(e) => (hashWindowPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (hashWindowPopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('hash_window')}
                    {/snippet}
                </Popover>
            </div>
            <div class="flex gap-4 items-center">
                <input
                    class="input"
                    type="number"
                    min="1"
                    bind:value={formData.hashwindow_value}
                    on:focus={() => openKeyboard('hashwindow_value')}
                />
                <Combobox
                    multiple={false}
                    data={hashUnitOptions}
                    defaultValue={[formData.hashwindow_unit]}
                    value={[formData.hashwindow_unit]}
                    onValueChange={(e) => (formData.hashwindow_unit = e.value[0])}
                    placeholder="Vyberte..."
                />
            </div>
        </label>

        <!-- Rozdělení souboru -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Rozdělení souboru</span>
                <Popover
                    open={splitPopover}
                    onOpenChange={(e) => (splitPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (splitPopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('split')}
                    {/snippet}
                </Popover>
            </div>
            <div class="flex gap-4 items-center">
                <input
                    class="input"
                    type="number"
                    min="1"
                    placeholder="whole"
                    bind:value={formData.split_value}
                    on:focus={() => openKeyboard('split_value')}
                />
                <Combobox
                    multiple={false}
                    data={splitUnitOptions}
                    defaultValue={[formData.split_unit]}
                    value={[formData.split_unit]}
                    onValueChange={(e) => (formData.split_unit = e.value[0])}
                    placeholder="Vyberte..."
                    disabled={isWholeSplit()}
                />
            </div>
        </label>

        <!-- vf (checkbox, disabled pokud splitValue != '' ) -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Ověření souboru (vf)</span>
                <Popover
                    open={vfPopover}
                    onOpenChange={(e) => (vfPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (vfPopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('vf')}
                    {/snippet}
                </Popover>
            </div>
            <label class="flex items-center">
                <input
                    type="checkbox"
                    class="checkbox"
                    on:change={toggleVF}
                    checked={formData.vf === 'on'}
                    disabled={!isWholeSplit()}
                />
                <span class="ml-2">Zapnout dodatečné ověření</span>
            </label>
        </label>

        <!-- diffwr -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Porovnání/změna bloků (diffwr)</span>
                <Popover
                    open={diffwrPopover}
                    onOpenChange={(e) => (diffwrPopover = e.open)}
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
                            <button class="btn-icon" on:click={() => (diffwrPopover = false)}>
                                <X />
                            </button>
                        </div>
                        {getExplanation('diffwr')}
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
        
        <!-- Notes -->
        <label class="label">
            <div class="flex items-center gap-2">
                <span>Poznámky</span>
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
    .select {
        padding: 0.5rem;
        border: 1px solid #ccc;
        border-radius: 4px;
        margin-top: 0.25rem;
    }
    .flex {
        display: flex;
    }
    .gap-4 {
        gap: 1rem;
    }
    .items-center {
        align-items: center;
    }
    .ml-2 {
        margin-left: 0.5rem;
    }
    .form-footer {
        margin-top: 2rem;
        display: flex;
        justify-content: flex-end;
    }
</style>
