<script lang="ts">
    import { getModalStore } from "@skeletonlabs/skeleton";
    import { invoke } from "@tauri-apps/api/core";

    const modalStore = getModalStore();

    // Výchozí hodnoty pro konfiguraci DCFLDD
    const formData = {
        confname: "", // Název konfigurace
        bs: "32768", // Výchozí velikost bloku (32 KB)
        count: "whole", // Počet bloků k získání
        ibs: "32768", // Velikost vstupního bloku
        obs: "32768", // Velikost výstupního bloku
        seek: "0", // Seek offset na začátku výstupu
        skip: "0", // Skip offset na začátku vstupu
        hash_types: ["md5"], // Výchozí hash (možnost více hashů)
        hashwindow: "4096", // Počet bajtů na hash okno
        hashlog: "", // Log soubor pro hash
        status: "on", // Zobrazovat status zprávy
        statusinterval: "256", // Interval status zpráv
        split: "ask", // Rozdělit soubory po určité velikosti
        splitformat: "nnn", // Formát rozdělení souboru
        vf: "ask", // Ověření souboru
        verifylog: "", // Log soubor pro ověření
    };

    async function onFormSubmit(): Promise<void> {
        try {
            await invoke("save_new_dcfldd_config", formData);
            console.log("Konfigurace DCFLDD byla uložena:", formData);
            modalStore.close();
        } catch (error) {
            console.error("Chyba při ukládání konfigurace DCFLDD:", error);
        }
    }

    function closeModal() {
        modalStore.close();
    }
</script>

{#if $modalStore[0]}
    <div class="modal-form">
        <div class="modal-content">
            <header class="modal-header">
                <h2>Nová konfigurace DCFLDD</h2>
            </header>

            <form class="modal-content" on:submit|preventDefault={onFormSubmit}>
                <label class="label">
                    <span>Název konfigurace</span>
                    <input class="input" type="text" bind:value={formData.confname} required />
                </label>

                <!-- Výběr velikosti bloku (bs) -->
                <label class="label">
                    <span>Velikost bloku (bs)</span>
                    <select class="select" bind:value={formData.bs}>
                        <option value="32768">32 KB (výchozí)</option>
                        <option value="65536">64 KB</option>
                        <option value="131072">128 KB</option>
                        <option value="262144">256 KB</option>
                        <option value="524288">512 KB</option>
                        <option value="1048576">1 MB</option>
                        <option value="1572864">1,5 MB</option>
                        <option value="2097152">2 MB</option>
                        <option value="3145728">3 MB</option>
                        <option value="4194304">4 MB</option>
                        <option value="5242880">5 MB</option>
                        <option value="6291456">6 MB</option>
                        <option value="7340032">7 MB</option>
                        <option value="8388608">8 MB</option>
                    </select>
                </label>

                <label class="label">
                    <span>Počet bloků k získání</span>
                    <select class="select" bind:value={formData.count}>
                        <option value="whole">Celý disk (výchozí)</option>
                        <option value="ask">Zeptat se</option>
                    </select>
                </label>

                <!-- Výběr více hashovacích algoritmů -->
                <label class="label">
                    <span>Vybrané hashovací algoritmy</span>
                    <div class="flex gap-4">
                        <label class="flex items-center">
                            <input type="checkbox" bind:group={formData.hash_types} value="md5" />
                            <span class="ml-2">MD5</span>
                        </label>
                        <label class="flex items-center">
                            <input type="checkbox" bind:group={formData.hash_types} value="sha1" />
                            <span class="ml-2">SHA1</span>
                        </label>
                        <label class="flex items-center">
                            <input type="checkbox" bind:group={formData.hash_types} value="sha256" />
                            <span class="ml-2">SHA256</span>
                        </label>
                        <label class="flex items-center">
                            <input type="checkbox" bind:group={formData.hash_types} value="sha384" />
                            <span class="ml-2">SHA384</span>
                        </label>
                        <label class="flex items-center">
                            <input type="checkbox" bind:group={formData.hash_types} value="sha512" />
                            <span class="ml-2">SHA512</span>
                        </label>
                    </div>
                </label>

                <label class="label">
                    <span>Počet bajtů na hash okno</span>
                    <input class="input" type="number" bind:value={formData.hashwindow} />
                </label>

                <label class="label">
                    <span>Zobrazovat status zprávy</span>
                    <select class="select" bind:value={formData.status}>
                        <option value="on">Zapnuto</option>
                        <option value="off">Vypnuto</option>
                    </select>
                </label>

                <label class="label">
                    <span>Interval status zpráv</span>
                    <input class="input" type="number" bind:value={formData.statusinterval} />
                </label>

                <label class="label">
                    <span>Rozdělení souborů po velikosti</span>
                    <select class="select" bind:value={formData.split}>
                        <option value="ask">Zeptat se</option>
                        <option value="disabled">Zakázáno</option>
                    </select>
                </label>

                <div class="modal-footer">
                    <button type="submit" class="btn btn-primary">Uložit</button>
                    <button type="button" class="btn btn-secondary" on:click={closeModal}>Zavřít</button>
                </div>
            </form>
        </div>
    </div>
{/if}

<style>
    .modal-form {
        width: 90vh;
        height: 90vh;
        overflow-y: auto;
        overscroll-behavior: contain;
        -webkit-overflow-scrolling: touch;
        contain: strict;
        backface-visibility: hidden;
        perspective: 1000px;
        background: white;
        border-radius: 10px;
        padding: 20px;
    }
    .modal-content {
        position: relative;
        z-index: 1;
        min-height: 100%;
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
    }
    .modal-footer {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
    }
</style>
