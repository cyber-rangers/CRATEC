<script lang="ts">
    import { Modal } from '@skeletonlabs/skeleton-svelte';
    import { TriangleAlert, CircleAlert, OctagonAlert } from 'lucide-svelte';

    export let warningData = { dco: 0, has_hpa: false, readable: true };
    export let openState: boolean = false;
    export let onResult: (result: boolean) => void = () => {};

    let showDco = false;
    let showHpa = false;

    $: {
        if (openState) {
            if (!warningData.readable) {
                showDco = false;
                showHpa = false;
            } else if (warningData.dco !== 0) {
                showDco = true;
                showHpa = false;
            } else if (warningData.has_hpa) {
                showDco = false;
                showHpa = true;
            } else {
                showDco = false;
                showHpa = false;
            }
        }
    }

    function modalClose(result = false) {
        openState = false;
        showDco = false;
        showHpa = false;
        onResult(result);
    }
    function onContinue() {
        if (showDco && warningData.has_hpa) {
            showDco = false;
            showHpa = true;
        } else {
            modalClose(true);
        }
    }
    function onCancel() {
        modalClose(false);
    }
</script>

<Modal
    open={openState}
    onOpenChange={(e) => (openState = e.open)}
    triggerBase=""
    contentBase={
        !warningData.readable
            ? "card modal-warning-modal bg-red-600 p-4 pt-2 w-[70vw] h-[80vh] space-y-6 shadow-xl relative"
            : showDco
                ? "card modal-warning-modal bg-gray-400 p-4 pt-2 w-[70vw] h-[80vh] space-y-6 shadow-xl relative"
                : showHpa
                    ? "card modal-warning-modal bg-yellow-400 p-4 pt-2 w-[70vw] h-[80vh] space-y-6 shadow-xl relative"
                    : "card modal-warning-modal p-4 pt-2 w-[70vw] h-[80vh] space-y-6 shadow-xl relative"
    }
    backdropClasses="backdrop-blur-sm"
>
    {#snippet content()}
        {#if !warningData.readable}
            <!-- ČERVENÉ VAROVÁNÍ: Disk je uzamčen -->
            <div class="flex flex-col items-center h-full justify-center">
                <OctagonAlert size={80} color="black" class="mb-4" />
                <h1 class="mb-2 text-center text-3xl font-bold text-black">Disk je uzamčen!</h1>
                <p class="mb-4 max-w-xl text-center text-base text-black font-semibold">
                    Disk je uzamčen na úrovni hardwaru nebo firmware a nelze z něj číst žádná data.
                </p>
                <p class="mb-8 max-w-xl text-center text-base text-black">
                    Tento stav znamená, že disk je chráněn proti čtení (například pomocí ATA password, SED, nebo jiného bezpečnostního mechanismu).
                    <br><br>
                    Akvizici není možné provést, protože disk neumožňuje přístup k datům. Pro další práci je nutné disk odemknout nebo použít jiný zdroj dat.
                </p>
            </div>
            <div class="absolute bottom-8 left-0 right-0 flex justify-center">
                <button
                    class="rounded bg-black px-8 py-3 font-semibold text-red-400 text-lg"
                    on:click={() => modalClose(false)}
                >
                    Ukončit
                </button>
            </div>
        {:else if showDco}
            <!-- ŠEDÉ VAROVÁNÍ: DCO -->
            <div class="flex flex-col items-center h-full justify-center">
                <CircleAlert size={160} color="black" class="mb-4" />
                <h1 class="mb-2 text-center text-3xl font-bold text-black">Detekována podpora DCO!</h1>
                <div class="mb-4 text-center">
                    <span class="block text-2xl font-extrabold text-black">LBA: {warningData.dco}</span>
                </div>
                <p class="mb-8 max-w-xl text-center text-base text-black">
                    <b>ZKONTROLUJTE, ZDA POČET LBA (SEKTORŮ) VÝŠE ODPOVÍDÁ HODNOTĚ NA ŠTÍTKU DISKU.</b><br>
                    Pokud nesouhlasí, některá data mohou být skrytá a akvizice nebude kompletní.
                </p>
            </div>
            <div class="absolute bottom-8 left-0 right-0 flex justify-between px-8">
                <button
                    class="rounded bg-black px-8 py-3 font-semibold text-gray-200 text-lg"
                    on:click={onCancel}
                >
                    Hodnoty nesedí
                </button>
                <button
                    class="rounded bg-black px-8 py-3 font-semibold text-gray-400 text-lg"
                    on:click={onContinue}
                >
                    Pokračovat
                </button>
            </div>
        {:else if showHpa}
            <!-- ŽLUTÉ VAROVÁNÍ: HPA -->
            <div class="flex flex-col items-center h-full justify-center">
                <TriangleAlert size={140} color="black" class="mb-4" />
                <h1 class="mb-2 text-center text-3xl font-bold text-black">HPA byl detekován!</h1>
                <p class="mb-8 max-w-xl text-center text-base text-black text-xl">
                    Byla detekována přítomnost HPA (Host Protected Area).<br>
                    HPA je oblast disku skrytá před operačním systémem.<br>
                    Pokud budete pokračovat v akvizici, tato oblast nebude zahrnuta do výsledných dat a některá data mohou chybět.
                </p>
            </div>
            <div class="absolute bottom-8 left-0 right-0 flex justify-between px-8">
                <button
                    class="rounded bg-black px-8 py-3 font-semibold text-gray-200 text-lg"
                    on:click={onCancel}
                >
                    Ukončit
                </button>
                <button
                    class="rounded bg-black px-8 py-3 font-semibold text-yellow-400 text-lg"
                    on:click={onContinue}
                >
                    I přesto pokračovat
                </button>
            </div>
        {/if}
    {/snippet}
</Modal>

<style>
    .modal-warning-modal {
        min-height: 500px;
        min-width: 600px;
        max-width: 900px;
        max-height: 90vh;
        box-sizing: border-box;
    }
</style>