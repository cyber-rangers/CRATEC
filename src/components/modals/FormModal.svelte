<!-- FormModal.svelte -->
<script lang="ts">
    import type { SvelteComponent } from 'svelte';
    import { getModalStore } from '@skeletonlabs/skeleton';

    export let parent: SvelteComponent;

    const modalStore = getModalStore();

    // Definice formulářových dat s výchozími hodnotami
    const formData = {
        source: '', // Zdrojový soubor nebo zařízení (povinné)
        A: 'ascii', // Kódová stránka (-A)
        b: '64', // Počet sektorů na čtení najednou (-b)
        B: 'whole', // Počet bajtů k získání (-B)
        c_method: 'deflate', // Metoda komprese (-c)
        c_level: 'none', // Úroveň komprese (-c)
        C: 'ask', // Číslo případu (-C)
        d: [], // Typy hashů (-d)
        D: 'ask', // Popis (-D)
        e: 'ask', // Jméno vyšetřovatele (-e)
        E: 'ask', // Číslo důkazu (-E)
        f: 'encase6', // Formát souboru EWF (-f)
        g: '', // Počet sektorů pro granularitu chyb (-g)
        l: '', // Název logovacího souboru (-l)
        m: 'fixed', // Typ média (-m)
        M: 'physical', // Příznaky média (-M)
        N: 'ask', // Poznámky (-N)
        o: '0', // Offset (-o)
        p: '', // Velikost bufferu procesu (-p)
        P: '512', // Bajtů na sektor (-P)
        q: false, // Tichý režim (-q)
        r: '2', // Počet opakování při chybě čtení (-r)
        R: false, // Obnovení akvizice (-R)
        s: false, // Přehodit bajtové páry (-s)
        S: '1.4 GiB', // Velikost segmentu souboru (-S)
        t: '', // Cílový soubor (-t)
        T: '', // TOC soubor (-T)
        u: false, // Bezobslužný režim (-u)
        v: false, // Podrobný výstup (-v)
        w: false, // Nulovat sektory při chybě čtení (-w)
        x: false, // Použít chunk data (-x)
        '2': '' // Sekundární cílový soubor (-2)
    };

    function onFormSubmit(): void {
        if ($modalStore[0].response) $modalStore[0].response(formData);
        modalStore.close();
    }

    // Základní třídy
    const cBase = 'card p-4 w-modal shadow-xl space-y-4';
    const cHeader = 'text-2xl font-bold';
    const cForm = 'space-y-4';
</script>

{#if $modalStore[0]}
    <div class="modal-form {cBase}">
        <header class={cHeader}>{$modalStore[0].title ?? 'Nastavení Příkazu'}</header>
        <article>{$modalStore[0].body ?? 'Vyplňte parametry příkazu.'}</article>
        <form class="{cForm}" on:submit|preventDefault={onFormSubmit}>
            <!-- Kódová stránka (-A) -->
            <label class="label">
                <span>Kódová stránka</span>
                <select class="select" bind:value={formData.A}>
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

            <!-- Počet sektorů na čtení (-b) -->
            <label class="label">
                <span>Počet sektorů na čtení</span>
                <select class="select" bind:value={formData.b}>
                    <option value="16">16</option>
                    <option value="32">32</option>
                    <option value="64">64 (výchozí)</option>
                    <option value="128">128</option>
                    <option value="256">256</option>
                    <option value="512">512</option>
                    <option value="1024">1024</option>
                    <option value="2048">2048</option>
                    <option value="4096">4096</option>
                    <option value="8192">8192</option>
                    <option value="16384">16384</option>
                    <option value="32768">32768</option>
                </select>
            </label>

            <!-- Počet bajtů k získání (-B) -->
        
            <label class="label">
                <span>Počet bajtů k získání</span>
                <select class="select" bind:value={formData.B}>
                    <option value="whole">celý disk (výchozí)</option>
                    <option value="ask">dotázat</option>   
                </select>
            </label>

            <!-- Komprese (-c) -->
            <label class="label">
                <span>Metoda komprese</span>
                <select class="select" bind:value={formData.c_method}>
                    <option value="deflate">deflate</option>
                </select>
            </label>
            <label class="label">
                <span>Úroveň komprese </span>
                <select class="select" bind:value={formData.c_level}>
                    <option value="none">none (výchozí)</option>
                    <option value="empty-block">empty-block</option>
                    <option value="fast">fast</option>
                    <option value="best">best</option>
                </select>
            </label>

            <!-- Číslo případu (-C) -->
            <label class="label">
                <span>Číslo případu</span>
                <select class="select" bind:value={formData.C}>
                    <option value="ask">dotázat (výchozí)</option>
                    <option value="empty">neuvádět</option>          
                </select> 
            </label>

            <!-- Typy hashů (-d) -->
            <label class="label">
                <span>Další typy hashů (MD5 vždy zahrnuto)</span>
                <div class="flex gap-4">
                    <label class="flex items-center">
                        <input type="checkbox" bind:group={formData.d} value="sha1" />
                        <span class="ml-2">SHA1</span>
                    </label>
                    <label class="flex items-center">
                        <input type="checkbox" bind:group={formData.d} value="sha256" />
                        <span class="ml-2">SHA256</span>
                    </label>
                </div>
            </label>

            <!-- Popis (-D) -->
            <label class="label">
                <span>Popis</span>
                <select class="select" bind:value={formData.D}>
                    <option value="ask">dotázat (výchozí)</option>
                    <option value="empty">neuvádět</option>          
                </select> 
            </label>

            <!-- Jméno vyšetřovatele (-e) -->
            <label class="label">
                <span>Jméno vyšetřovatele</span>
                <select class="select" bind:value={formData.e}>
                    <option value="ask">dotázat (výchozí)</option>
                    <option value="empty">neuvádět</option>          
                </select> 
            </label>

            <!-- Číslo důkazu (-E) -->
            <label class="label">
                <span>Číslo důkazu</span>
                <select class="select" bind:value={formData.E}>
                    <option value="ask">dotázat (výchozí)</option>
                    <option value="empty">neuvádět</option>          
                </select> 
            </label>

            <!-- Formát souboru (-f) -->
            <label class="label">
                <span>Formát souboru EWF</span>
                <select class="select" bind:value={formData.f}>
                    <option value="ewf">ewf</option>
                    <option value="smart">smart</option>
                    <option value="ftk">ftk</option>
                    <option value="encase2">encase2</option>
                    <option value="encase3">encase3</option>
                    <option value="encase4">encase4</option>
                    <option value="encase5">encase5</option>
                    <option value="encase6">encase6 (výchozí)</option>
                    <option value="encase7">encase7</option>
                    <option value="encase7-v2">encase7-v2</option>
                    <option value="linen5">linen5</option>
                    <option value="linen6">linen6</option>
                    <option value="linen7">linen7</option>
                    <option value="ewfx">ewfx</option>
                </select>
            </label>

            <!-- Příznaky média (-M) -->
            <label class="label">
                <span>Příznaky média</span>
                <select class="select" bind:value={formData.M}>
                    <option value="physical">physical (výchozí)</option>
                    <option value="logical">logical</option>
                </select>
            </label>

            <!-- Poznámky (-N) -->
            <label class="label">
                <span>Poznámky</span>
                <select class="select" bind:value={formData.N}>
                    <option value="ask">dotázat (výchozí)</option>
                    <option value="empty">neuvádět</option>          
                </select> 
            </label>

            <!-- Offset (-o) -->
            <label class="label">
                <span>Offset</span>
                <select class="select" bind:value={formData.o}>
                    <option value="0">0 (výchozí)</option>
                    <option value="ask">dotázat</option>   
                </select>
            </label>

            <!-- Velikost bufferu procesu (-p) -->
            <label class="label">
                <span>Velikost bufferu procesu</span>
                <input class="input" type="text" bind:value={formData.p} placeholder="Výchozí je velikost chunku" />
            </label>

            <!-- Bajtů na sektor (-P) -->
            <label class="label">
                <span>Bajtů na sektor </span>
                <input class="input" type="number" bind:value={formData.P} placeholder="512" />
            </label>

            <!-- Tichý režim (-q) -->
            <label class="flex items-center">
                <input type="checkbox" bind:checked={formData.q} />
                <span class="ml-2">Tichý režim </span>
            </label>

            <!-- Počet opakování při chybě čtení (-r) -->
            <label class="label">
                <span>Počet opakování při chybě čtení </span>
                <input class="input" type="number" bind:value={formData.r} placeholder="2" />
            </label>

            <!-- Obnovení akvizice (-R) -->
            <label class="flex items-center">
                <input type="checkbox" bind:checked={formData.R} />
                <span class="ml-2">Obnovení akvizice</span>
            </label>

            <!-- Přehodit bajtové páry (-s) -->
            <label class="flex items-center">
                <input type="checkbox" bind:checked={formData.s} />
                <span class="ml-2">Přehodit bajtové páry (-s)</span>
            </label>

            <!-- Velikost segmentu souboru (-S) -->
            <label class="label">
                <span>Velikost segmentu souboru </span>
                <input class="input" type="text" bind:value={formData.S} placeholder="1.4 GiB" />
            </label>

            <!-- Cílový soubor (-t) -->
            <label class="label">
                <span>Cílový soubor </span>
                <input class="input" type="text" bind:value={formData.t} placeholder="Bez přípony" />
            </label>

            <!-- TOC soubor (-T) -->
            <label class="label">
                <span>TOC soubor</span>
                <input class="input" type="text" bind:value={formData.T} placeholder="CUE formát" />
            </label>

            <!-- Bezobslužný režim (-u) -->
            <label class="flex items-center">
                <input type="checkbox" bind:checked={formData.u} />
                <span class="ml-2">Bezobslužný režim</span>
            </label>

            <!-- Podrobný výstup (-v) -->
            <label class="flex items-center">
                <input type="checkbox" bind:checked={formData.v} />
                <span class="ml-2">Podrobný výstup </span>
            </label>

            <!-- Nulovat sektory při chybě čtení (-w) -->
            <label class="flex items-center">
                <input type="checkbox" bind:checked={formData.w} />
                <span class="ml-2">Nulovat sektory při chybě čtení </span>
            </label>

            <!-- Použít chunk data (-x) -->
            <label class="flex items-center">
                <input type="checkbox" bind:checked={formData.x} />
                <span class="ml-2">Použít chunk data</span>
            </label>

            <!-- Sekundární cílový soubor (-2) -->
            <label class="label">
                <span>Sekundární cílový soubor</span>
                <input class="input" type="text" bind:value={formData['2']} placeholder="Bez přípony" />
            </label>

            <!-- Tlačítka formuláře -->
            <footer class="modal-footer {parent.regionFooter}">
                <button class="btn {parent.buttonNeutral}" type="button" on:click={parent.onClose}>{parent.buttonTextCancel}</button>
                <button class="btn {parent.buttonPositive}" type="submit">{parent.buttonTextSubmit}</button>
            </footer>
        </form>
    </div>
{/if}

<style>
    .label {
        display: flex;
        flex-direction: column;
        margin-bottom: 1rem;
    }
    .input, .select {
        padding: 0.5rem;
        border: 1px solid #ccc;
        border-radius: 4px;
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
    .modal-footer {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
    }
</style>
