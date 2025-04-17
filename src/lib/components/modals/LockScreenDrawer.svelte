<script lang="ts">
    import { Modal } from '@skeletonlabs/skeleton-svelte';
    import { SquareX, Send, Lock, LockOpen } from 'lucide-svelte';
    import { Resource, invoke } from '@tauri-apps/api/core';
    import { Toaster, createToaster } from '@skeletonlabs/skeleton-svelte';

    export let isOpen: boolean = false;

    let input: string = '';
    let firstInput: string | null = null;
    let locked: boolean = false;

    const toaster = createToaster({
        placement: 'top'
    });

    function closeDrawer() {
        isOpen = false;
        reset();
    }

    function reset() {
        input = '';
        firstInput = null;
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
            if (firstInput === null) {
                if (input.length < 4) {
                    toaster.error({ title: 'Zadejte minimálně 4 číslice!' });
                    return;
                }
                firstInput = input;
                input = '';
                toaster.info({ title: 'Potvrďte kód!' });
                return;
            }
            if (input !== firstInput) {
                toaster.error({ title: 'Kódy se neshodují!' });
                firstInput = null;
                input = '';
                return;
            }
            await invoke('lock_system', { code: input });
            locked = true;
            reset();
            toaster.success({ title: 'Systém byl úspěšně zamčen!' });
        } else {
            const ok: boolean = await invoke('unlock_system', { code: input });
            if (ok) {
                locked = false;
                isOpen = false;
                reset();
                toaster.success({ title: 'Systém odemčen!' });
            } else {
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
            <!-- Levá polovina (vstup, logo a numpad) -->
            <div class="border-primary-8100 flex w-1/2 flex-col items-center justify-center border-r-2">
                <!-- Logo zámku nad vstupem - změněna barva na primary-500 -->
                <div class="mb-8 flex items-center justify-center">
                    {#if locked}
                        <Lock class="w-12 h-12 text-primary-500" />
                    {:else}
                        <LockOpen class="w-12 h-12 text-primary-500" />
                    {/if}
                </div>
                <!-- Vstup -->
                <div class="mb-6 flex flex-col items-center">
                    <input
                        type="password"
                        class="input preset-outlined-primary-500 h-12 w-55 rounded px-3 py-2 text-lg"
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
                </div>
                <!-- Numpad -->
                <div class="grid grid-cols-3 gap-4">
                    {#each ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as num}
                        <button
                            class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
                            on:click={() => handleDigit(num)}
                        >
                            {num}
                        </button>
                    {/each}
                    <button
                        class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
                        on:click={handleClear}
                    >
                        <SquareX />
                    </button>
                    <button
                        class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
                        on:click={() => handleDigit('0')}
                    >
                        0
                    </button>
                    <button
                        class="btn preset-filled-primary-500 h-16 w-16 font-bold text-white shadow-md"
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
            var(--color-surface-600)
        );
    }
</style>
