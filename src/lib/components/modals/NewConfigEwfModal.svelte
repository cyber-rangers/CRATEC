<script lang="ts">
  import type { SvelteComponent } from "svelte";
  import { getModalStore } from "@skeletonlabs/skeleton";
  import VirtualKeyboard from "../VirtualKeyboard.svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let parent: SvelteComponent;

  const modalStore = getModalStore();
  let showKeyboard = false;
  let activeInput: string;

  // Definice formulářových dat s výchozími hodnotami
  const formData = {
    confname: "", // Název configu
    codepage: "ascii", // Kódová stránka (-A)
    sectors_per_read: "64", // Počet sektorů na čtení najednou (-b)
    bytes_to_read: "whole", // Počet bajtů k získání (-B)
    compression_method: "deflate", // Metoda komprese (-c)
    compression_level: "none", // Úroveň komprese (-c)
    hash_types: [], // Typy hashů (-d)
    ewf_format: "encase6", // Formát souboru EWF (-f)
    granularity_sectors: 2, // Počet sektorů pro granularitu chyb (-g)
    notes: "ask", // Poznámky (-N)
    offset: "0", // Offset (-o)
    process_buffer_size: "auto", // Velikost bufferu procesu (-p)
    bytes_per_sector: "auto", // Bajtů na sektor (-P)
    quiet_mode: false, // Tichý režim (-q)
    read_retry_count: "2", // Počet opakování při chybě čtení (-r)
    swap_byte_pairs: false, // Přehodit bajtové páry (-s)
    segment_size: "1.4 GiB", // Velikost segmentu souboru (-S)
    verbose_output: false, // Podrobný výstup (-v)
    zero_on_read_error: false, // Nulovat sektory při chybě čtení (-w)
    use_chunk_data: false, // Použít chunk data (-x)
  };

  async function onFormSubmit(): Promise<void> {
    try {
      // Zavoláme Rust funkci 'save_new_config_command' a předáme celé formData
      const result = await invoke("save_new_ewf_config", formData);
      console.log("Formulář odeslán a data uložena, výsledek:", result);
      // Předáme data callbacku, pokud je definován
      if ($modalStore[0].response) $modalStore[0].response(formData);
      modalStore.close();
      showKeyboard = false;
    } catch (error) {
      console.error("Chyba při odesílání formuláře:", error);
    }
  }

  // Při focusu na vstupním poli zobrazíme virtuální klávesnici a uložíme jméno pole
  function onInputFocus(event: Event, fieldName: string) {
    showKeyboard = true;
    activeInput = fieldName;
  }

  // Základní třídy pro stylování
  const cBase = "card p-4 w-modal shadow-xl space-y-4";
  const cHeader = "text-2xl font-bold";
  const cForm = "space-y-4";
</script>

{#if $modalStore[0]}
  {@const modal = $modalStore[0]}

  <div class="modal-form {cBase}">
    <div class="modal-content">
      <header class={cHeader}>
        {$modalStore[0].title ?? "Nastavení Příkazu"}
      </header>
      <article>
        {$modalStore[0].body ?? "Vyplňte parametry příkazu."}
      </article>

      <form class={cForm} on:submit|preventDefault={onFormSubmit}>
        <label class="label">
          <span>Název konfigurace</span>
          <input
            class="input"
            name="confname"
            type="text"
            on:focus={(e) => onInputFocus(e, "confname")}
            required
          />
        </label>

        <!-- Kódová stránka (-A) -->
        <label class="label">
          <span>Kódová stránka</span>
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

        <!-- Počet sektorů na čtení (-b) -->
        <label class="label">
          <span>Počet sektorů na čtení</span>
          <select
            class="select"
            name="sectors_per_read"
            bind:value={formData.sectors_per_read}
          >
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
          <select
            class="select"
            name="bytes_to_read"
            bind:value={formData.bytes_to_read}
          >
            <option value="whole">celý disk (výchozí)</option>
            <option value="ask">dotázat</option>
          </select>
        </label>

        <!-- Komprese (-c) -->
        <label class="label">
          <span>Metoda komprese</span>
          <select
            class="select"
            name="compression_method"
            bind:value={formData.compression_method}
          >
            <option value="deflate">deflate</option>
          </select>
        </label>
        <label class="label">
          <span>Úroveň komprese</span>
          <select
            class="select"
            name="compression_level"
            bind:value={formData.compression_level}
          >
            <option value="none">none (výchozí)</option>
            <option value="empty-block">empty-block</option>
            <option value="fast">fast</option>
            <option value="best">best</option>
          </select>
        </label>

        <!-- Typy hashů (-d) -->
        <label class="label">
          <span>Další typy hashů (MD5 vždy zahrnuto)</span>
          <div class="flex gap-4">
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:group={formData.hash_types}
                value="sha1"
              />
              <span class="ml-2">SHA1</span>
            </label>
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:group={formData.hash_types}
                value="sha256"
              />
              <span class="ml-2">SHA256</span>
            </label>
          </div>
        </label>

        <!-- Formát souboru (-f) -->
        <label class="label">
          <span>Formát souboru EWF</span>
          <select
            class="select"
            name="ewf_format"
            bind:value={formData.ewf_format}
          >
            <option value="ewf">ewf</option>
            <option value="smart">smart</option>
            <option value="ftk">ftk</option>
            <option value="encase2">encase2</option>
            <option value="encase3">encase3</option>
            <option value="encase4">encase4</option>
            <option value="encase5">encase5</option>
            <option value="encase6">encase6 (výchozí)</option>
            <option value="linen5">linen5</option>
            <option value="linen6">linen6</option>
            <option value="ewfx">ewfx</option>
          </select>
        </label>

        <!-- Počet sektorů pro granularitu chyb (-g) -->
        <label class="label">
          <span>Počet opakování při chybě čtení</span>
          <input
            class="input"
            name="granularity_sectors"
            type="number"
            bind:value={formData.granularity_sectors}
            on:focus={(e) => onInputFocus(e, "granularity_sectors")}
            placeholder="2"
            required
          />
        </label>

        <!-- Poznámky (-N) -->
        <label class="label">
          <span>Poznámky</span>
          <select class="select" name="notes" bind:value={formData.notes}>
            <option value="ask">dotázat (výchozí)</option>
            <option value="empty">neuvádět</option>
          </select>
        </label>

        <!-- Offset (-o) -->
        <label class="label">
          <span>Offset</span>
          <select class="select" name="offset" bind:value={formData.offset}>
            <option value="0">0 (výchozí)</option>
            <option value="ask">dotázat</option>
          </select>
        </label>

        <!-- Velikost bufferu procesu (-p) -->
        <label class="label">
          <span>Velikost bufferu procesu</span>
          <select
            class="select"
            name="process_buffer_size"
            bind:value={formData.process_buffer_size}
          >
            <option value="auto">Automaticky detekovat (výchozí)</option>
            <option value="128">128</option>
            <option value="256">256</option>
            <option value="512">512</option>
            <option value="1024">1024</option>
            <option value="2048">2048</option>
            <option value="4096">4096</option>
            <option value="8192">8192</option>
            <option value="16384">16384</option>
          </select>
        </label>

        <!-- Bajtů na sektor (-P) -->
        <label class="label">
          <span>Bajtů na sektor</span>
          <select
            class="select"
            name="bytes_per_sector"
            bind:value={formData.bytes_per_sector}
          >
            <option value="auto">Automaticky detekovat (výchozí)</option>
            <option value="128">128</option>
            <option value="256">256</option>
            <option value="512">512</option>
            <option value="1024">1024</option>
            <option value="2048">2048</option>
            <option value="4096">4096</option>
            <option value="8192">8192</option>
            <option value="16384">16384</option>
          </select>
        </label>

        <!-- Tichý režim (-q) -->
        <label class="flex items-center">
          <input
            type="checkbox"
            name="quiet_mode"
            bind:checked={formData.quiet_mode}
          />
          <span class="ml-2">Tichý režim</span>
        </label>

        <!-- Počet opakování při chybě čtení (-r) -->
        <label class="label">
          <span>Počet opakování při chybě čtení</span>
          <input
            class="input"
            name="read_retry_count"
            type="number"
            bind:value={formData.read_retry_count}
            on:focus={(e) => onInputFocus(e, "read_retry_count")}
            placeholder="2"
          />
        </label>

        <!-- Přehodit bajtové páry (-s) -->
        <label class="flex items-center">
          <input
            type="checkbox"
            name="swap_byte_pairs"
            bind:checked={formData.swap_byte_pairs}
          />
          <span class="ml-2">Přehodit bajtové páry (-s)</span>
        </label>

        <!-- Velikost segmentu souboru (-S) -->
        <label class="label">
          <span>Velikost segmentu souboru (hodnota od 1.0 MiB do 7.9 EiB)</span>
          <input
            class="input"
            name="segment_size"
            type="text"
            bind:value={formData.segment_size}
            on:focus={(e) => onInputFocus(e, "segment_size")}
            placeholder="1.4 GiB"
            required
          />
        </label>


        <!-- Podrobný výstup (-v) -->
        <label class="flex items-center">
          <input
            type="checkbox"
            name="verbose_output"
            bind:checked={formData.verbose_output}
          />
          <span class="ml-2">Podrobný výstup</span>
        </label>

        <!-- Nulovat sektory při chybě čtení (-w) -->
        <label class="flex items-center">
          <input
            type="checkbox"
            name="zero_on_read_error"
            bind:checked={formData.zero_on_read_error}
          />
          <span class="ml-2">Nulovat sektory při chybě čtení</span>
        </label>

        <!-- Použít chunk data (-x) -->
        <label class="flex items-center">
          <input
            type="checkbox"
            name="use_chunk_data"
            bind:checked={formData.use_chunk_data}
          />
          <span class="ml-2">Použít chunk data</span>
        </label>

        <!-- Pouze tlačítko Submit – modal se zavře pouze při odeslání formuláře -->
        <footer class="modal-footer {parent.regionFooter}">
          <button class="btn {parent.buttonPositive}" type="submit">
            {parent.buttonTextSubmit}
          </button>
        </footer>
      </form>
    </div>
  </div>

  <!-- Zobrazení virtuální klávesnice -->
  {#if showKeyboard}
    <VirtualKeyboard
      {activeInput}
      {formData}
      on:closeKeyboard={() => (showKeyboard = false)}
    />
  {/if}
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
  }
  .modal-form::-webkit-scrollbar {
    width: 1px;
    background-color: transparent;
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
