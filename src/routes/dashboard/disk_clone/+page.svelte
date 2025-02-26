<script lang="ts">
    import { getModalStore } from "@skeletonlabs/skeleton";
    import type { ModalSettings } from "@skeletonlabs/skeleton";
    import { FaUsb, FaHdd } from "svelte-icons/fa";
    import { copyRunStore } from "$lib/stores/copyRunStore";

    //Stepper
    // Source Data
    const steps = [
        { label: "Step 1", description: "The description of step 1." },
        { label: "Step 2", description: "The description of step 2." },
        { label: "Step 3", description: "The description of step 3." },
        { label: "Step 4", description: "The description of step 4." },
        { label: "Step 5", description: "The description of step 5." },
    ];

    // Reactive
    let currentStep = $state(0);
    const isFirstStep = $derived(currentStep === 0);
    const isLastStep = $derived(currentStep === steps.length - 1);

    /** Determine if on the current step. */
    function isCurrentStep(index: number) {
        return currentStep === index;
    }

    /** Jump to a particular step. */
    function setStep(index: number) {
        currentStep = index;
    }

    /** Progress to the previous step. */
    function prevStep() {
        currentStep--;
    }

    /** Progress to the next step. */
    function nextStep() {
        currentStep++;
    }

    const modalStore = getModalStore();

    const diskModal: ModalSettings = {
        type: "component",
        component: "DiskSelectModal",
        title: "Vyberte disk",
    };
</script>

<div class="w-full">
    <!-- Stepper -->
    <div class="space-y-8">
        <!-- Timeline -->
        <div class="relative">
            <!-- Numerals -->
            <div class="flex justify-between items-center gap-4">
                {#each steps as step, i}
                    <!-- Numeral Button -->
                    <button
                        class="btn-icon btn-icon-sm rounded-full {isCurrentStep(
                            i,
                        )
                            ? 'preset-filled-primary-500'
                            : 'preset-filled-surface-200-800'}"
                        onclick={() => setStep(i)}
                        title={step.label}
                    >
                        <span class="font-bold">{i + 1}</span>
                    </button>
                {/each}
            </div>
            <!-- Line -->
            <hr
                class="hr !border-surface-200-800 absolute top-[50%] left-0 right-0 z-[-1]"
            />
        </div>
        <!-- Loop all steps -->

        <!-- Filter to current step only -->
        {#if currentStep === 0}
            <!-- Individual steps -->
            <div
                class="card bg-surface-700-900 p-10 space-y-2 text-center"
                style="height: 390px;"
            >
                <h2 class="h3"></h2>
                <p></p>
            </div>
        {/if}
        {#if currentStep === 1}
            <div
                class="card bg-surface-700-900 p-10 space-y-2 text-center"
                style="height: 390px;"
            >
                <div class="container">
                    <!-- Levá sekce (VSTUP) -->
                    <div class="section left">
                        <div class="header">VSTUP</div>
                        <div class="content">
                            {#if $copyRunStore.inputDisk}
                                <button
                                    class="box"
                                    type="button"
                                    onclick={() =>
                                        modalStore.trigger({
                                            ...diskModal,
                                            meta: { side: "input" },
                                        })}
                                >
                                    {#if $copyRunStore.inputDisk.type === "usb"}
                                        <div
                                            class="connected-icon"
                                            style="width: 20px;"
                                        >
                                            <FaUsb />
                                        </div>
                                    {:else}
                                        <div
                                            class="connected-icon"
                                            style="width: 20px;"
                                        >
                                            <FaHdd />
                                        </div>
                                    {/if}
                                    <span
                                        >{$copyRunStore.inputDisk
                                            .interface}</span
                                    >
                                </button>
                            {:else}
                                <button
                                    class="box"
                                    type="button"
                                    onclick={() =>
                                        modalStore.trigger({
                                            ...diskModal,
                                            meta: { side: "input" },
                                        })}
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

                    <!-- Pravá sekce (VYSTUP) -->
                    <div class="section right">
                        <div class="header">VÝSTUP</div>
                        <div class="content">
                            <div class="output-list">
                                {#each $copyRunStore.outputDisks as disk (disk.interface)}
                                    <div class="output-item">
                                        {#if disk.type === "usb"}
                                            <div
                                                class="connected-icon"
                                                style="width: 20px;"
                                            >
                                                <FaUsb />
                                            </div>
                                        {:else}
                                            <div
                                                class="connected-icon"
                                                style="width: 20px;"
                                            >
                                                <FaHdd />
                                            </div>
                                        {/if}
                                        <span>{disk.interface}</span>
                                    </div>
                                {/each}
                                <!-- Tlačítko pro přidání dalšího výstupního disku -->
                                <button
                                    class="box small"
                                    type="button"
                                    onclick={() =>
                                        modalStore.trigger({
                                            ...diskModal,
                                            meta: { side: "output" },
                                        })}
                                >
                                    +
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        {/if}
        <!-- Navigation -->
        <nav class="flex justify-between items-center gap-4">
            <!-- Back Button -->
            <button
                type="button"
                class=" hover:preset-filled"
                onclick={prevStep}
                disabled={isFirstStep}
            >
                
                <span>Předchozí</span>
            </button>
            <!-- Next Button -->
            <button
                type="button"
                class="hover:preset-filled"
                onclick={nextStep}
                disabled={isLastStep}
            >
                <span>Další</span>
                
            </button>
        </nav>
    </div>
</div>

<style lang="postcss">
    /* Nastavení pozadí stránky */
    body {
        background-color: #322b2b;
    }

    .container {
        display: flex;
        width: 100%;
        height: 100%; /* Celá výška okna */
        overflow: hidden;
    }

    .section {
        flex: 1;
        position: relative;
        display: flex;
        flex-direction: column;
    }

    /* Nadpisy (VSTUP/VYSTUP) – umístěné nahoře */
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

    /* Obsah vycentrovaný ve zbývajícím prostoru sekce */
    .content {
        flex: 1;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    /* Boxy pro vstup a tlačítko pro přidání výstupu */
    .box {
        width: 80px;
        height: 80px;
        border: 2px solid #ccc;
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

    /* Seznam výstupních disků */
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
        border: 1px solid #ccc;
        border-radius: 4px;
        font-size: 0.9rem;
        color: white;
    }

    /* Animovaná šipka ve střední sekci – šipka směřuje vodorovně zleva doprava */
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
        border-bottom: 15px solid rgba(var(--color-primary-500) / 1);
        border-right: 15px solid rgba(var(--color-primary-500) / 1);
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
</style>
