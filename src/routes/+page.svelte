<script>
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { writable } from "svelte/store";
    import { TriangleAlert } from 'lucide-svelte';

    const integrityOk = writable(/** @type {boolean|null} */(null));
    const aideJson = writable(""); // pro výstup JSONu
    const showModal = writable(false);

    onMount(async () => {
        try {
            const result = await invoke("run_aide_check_json");
            // Ulož pouze čistý JSON z Rustu
            aideJson.set(result.raw_json ?? JSON.stringify(result, null, 2));
            if (result.anything_changed) {
                integrityOk.set(false);
            } else {
                integrityOk.set(true);
                goto("/dashboard/disk_clone");
            }
        } catch (e) {
            integrityOk.set(false);
            aideJson.set(e && e.toString ? e.toString() : String(e));
        }
    });

    function openModal() {
        showModal.set(true);
    }
    function closeModal() {
        showModal.set(false);
    }
</script>

{#if $integrityOk === false}
    <div class="alert-screen">
        <TriangleAlert size="240" color="white" class="alert-icon" />
        <h1>Manipulace se systémem!</h1>
        <p>Byly detekovány změny v systému. Zařízení není bezpečné nadále využívat!</p>
        <button class="show-errors-btn" on:click={openModal}>Zobrazit chyby</button>
    </div>
{/if}

{#if $showModal}
    <div class="modal-backdrop" on:click={closeModal}>
        <button class="close-x" on:click|stopPropagation={closeModal} aria-label="Zavřít">&times;</button>
        <pre class="modal-pre" on:click|stopPropagation>{$aideJson}</pre>
    </div>
{/if}

{#if $integrityOk !== false}
    <div class="h-screen flex flex-col items-center justify-center relative">
        <img src="/rangers-logo.png" alt="Rangers Logo" class="logo fade-text" />
        <h2 class="fade-text">
            <span
                class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone"
            >
                CRATEC
            </span>
        </h2>
    </div>
{/if}

<style lang="postcss">
    @keyframes fade {
        0% {
            opacity: 1;
        }
        50% {
            opacity: 0.3;
        }
        100% {
            opacity: 1;
        }
    }

    .h-screen {
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
    }

    h2 {
        font-weight: bold;
        font-size: 5rem;
        margin-bottom: 75px;
    }

    .logo {
        width: 350px;
        height: auto;
    }

    .fade-text {
        animation: fade 2s infinite;
    }

    .alert-screen {
        background: #b91c1c;
        color: white;
        position: fixed;
        inset: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        z-index: 9999;
    }
    .alert-icon {
        margin-bottom: 2rem;
    }
    .alert-screen h1 {
        font-size: 3rem;
        font-weight: bold;
        margin-bottom: 1rem;
    }
    .alert-screen p {
        font-size: 1.5rem;
    }

    .show-errors-btn {
        margin-top: 2rem;
        padding: 1rem 2rem;
        font-size: 1.2rem;
        background: #fff;
        color: #b91c1c;
        border: none;
        border-radius: 0.5rem;
        cursor: pointer;
        font-weight: bold;
        transition: background 0.2s;
    }
    .show-errors-btn:hover {
        background: #fca5a5;
    }
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0,0,0,0.85);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 10000;
    }
    .modal-pre {
        background: #111;
        color: #fff;
        padding: 0;
        border-radius: 0;
        width: 98vw;
        height: 96vh;
        max-width: 98vw;
        max-height: 96vh;
        overflow: auto;
        font-size: 1.1rem;
        box-sizing: border-box;
        margin: 0;
    }
    .close-x {
        position: absolute;
        top: 2vh;
        right: 2vw;
        width: 3rem;
        height: 3rem;
        border-radius: 50%;
        border: none;
        background: #b91c1c;
        color: #fff;
        font-size: 2rem;
        font-weight: bold;
        cursor: pointer;
        z-index: 10001;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background 0.2s;
    }
    .close-x:hover {
        background: #ef4444;
    }
</style>
