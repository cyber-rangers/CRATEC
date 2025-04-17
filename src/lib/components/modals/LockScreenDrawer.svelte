<script lang="ts">
    import { Modal } from '@skeletonlabs/skeleton-svelte';
    import { SquareX, Send } from 'lucide-svelte';
    import { Resource, invoke } from '@tauri-apps/api/core';
    import { Toaster, createToaster } from '@skeletonlabs/skeleton-svelte';

    export let isOpen: boolean = false;

    let input: string = '';
    let firstInput: string | null = null;
    let locked: boolean = false;
    let error: string = '';

    // Přidáme Toaster
    const toaster = createToaster();

    function closeDrawer() {
        isOpen = false;
        reset();
    }

    function reset() {
        input = '';
        firstInput = null;
        error = '';
    }

    function openDrawer() {
        isOpen = true;
    }

    function handleDigit(d: string) {
        if (input.length < 6) input += d;
    }

    function handleClear() {
        input = '';
    }

    async function handleSubmit() {
        if (!locked) {
            // První zadání kódu
            if (firstInput === null) {
                if (input.length < 4) {
                    error = 'Minimálně 4 číslice';
                    // Ukážeme chybu i jako toast
                    toaster.error({ title: 'Zadejte minimálně 4 číslice!' });
                    return;
                }
                firstInput = input;
                input = '';
                error = 'Potvrďte kód';
                toaster.info({ title: 'Potvrďte kód!' });
                return;
            }
            // Druhé zadání – potvrzení
            if (input !== firstInput) {
                error = 'Kódy se neshodují';
                toaster.error({ title: 'Kódy se neshodují!' });
                firstInput = null;
                input = '';
                return;
            }
            // Uzamčení
            await invoke('lock_system', { code: input });
            locked = true;
            reset();
            toaster.success({ title: 'Systém byl úspěšně zamčen!' });
        } else {
            // Odemčení
            const ok: boolean = await invoke('unlock_system', { code: input });
            if (ok) {
                locked = false;
                isOpen = false; // Odemčené okno můžeme zavřít
                reset();
                toaster.success({ title: 'Systém odemčen!' });
            } else {
                error = 'Neplatný kód';
                toaster.error({ title: 'Neplatný kód!' });
                input = '';
            }
        }
    }
</script>

<Modal
    open={isOpen}
    onOpenChange={(e) => (isOpen = e.open)}
    triggerBase="btn preset-tonal"
    contentBase="preset-gradient-one h-screen w-screen p-4 flex flex-col"
    positionerJustify="justify-start"
    positionerAlign="items-start"
    positionerPadding=""
    transitionsPositionerIn={{ y: -1000, duration: 200 }}
    transitionsPositionerOut={{ y: -1000, duration: 200 }}
>
    {#snippet trigger()}
        Open Fullscreen Drawer
    {/snippet}

    {#snippet content()}
        <div class="flex h-full w-full flex-row">
            <!-- Levá polovina (vstup a numpad) -->
            <div class="border-primary-8100 flex w-1/2 flex-col items-center justify-center border-r-2">
                <!-- Vstup -->
                <div class="mb-6 flex flex-col items-center">
                    <input
                        type="password"
                        class="input border-primary-500 h-12 w-55 rounded border-2 px-3 py-2 text-lg"
                        bind:value={input}
                        readonly
                        placeholder={
                            locked
                                ? 'Zadejte kód odemčení'
                                : firstInput
                                ? 'Potvrďte kód'
                                : 'Zadejte kód'
                        }
                    />
                    {#if error}
                        <div class="text-red-500 text-sm mt-1">{error}</div>
                    {/if}
                </div>
                <!-- Numpad -->
                <div class="grid grid-cols-3 gap-4">
                    {#each ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as num}
                        <button
                            class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white"
                            on:click={() => handleDigit(num)}
                        >
                            {num}
                        </button>
                    {/each}
                    <button
                        class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white"
                        on:click={handleClear}
                    >
                        <SquareX />
                    </button>
                    <button
                        class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white"
                        on:click={() => handleDigit('0')}
                    >
                        0
                    </button>
                    <button
                        class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white"
                        on:click={handleSubmit}
                    >
                        <Send />
                    </button>
                </div>
            </div>

            <!-- Pravá polovina (zavírací tlačítko) -->
            <div class="flex w-1/2 items-center justify-center">
                {#if !locked}
                    <button class="btn preset-filled mt-4" on:click={closeDrawer}>
                        Zavřít
                    </button>
                {/if}
            </div>
        </div>
    {/snippet}
</Modal>

<!-- Toaster -->
<Toaster {toaster}></Toaster>

<style>
    :global(.preset-gradient-one) {
        background-image: linear-gradient(
            45deg,
            var(--color-surface-900),
            var(--color-primary-900)
        );
    }
</style>
