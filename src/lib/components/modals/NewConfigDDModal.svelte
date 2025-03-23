<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import VirtualKeyboard from "../VirtualKeyboard.svelte";
	import { createEventDispatcher } from 'svelte';
  
	// Řízení zobrazení modalu přes prop open
	export let open: boolean = false;
	const dispatch = createEventDispatcher();
  
	let formData = {
	  confname: '',       // Název konfigurace
	  bs: '32768',        // Výchozí velikost bloku (32 KB)
	  count: 'whole',     // Počet bloků k získání
	  ibs: '32768',       // Velikost vstupního bloku
	  obs: '32768',       // Velikost výstupního bloku
	  seek: '0',          // Seek offset na začátku výstupu
	  skip: '0',          // Skip offset na začátku vstupu
	  hash_types: ['md5'],// Výchozí hash (možnost více hashů)
	  hashwindow: '4096', // Počet bajtů na hash okno
	  hashlog: '',        // Log soubor pro hash
	  status: 'on',       // Zobrazovat status zprávy
	  statusinterval: '256', // Interval status zpráv
	  split: 'ask',       // Rozdělit soubory po určité velikosti
	  splitformat: 'nnn', // Formát rozdělení souboru
	  vf: 'ask',          // Ověření souboru
	  verifylog: '',      // Log soubor pro ověření
  
	  // NOVÉ PARAMETRY:
	  conv: 'noerror,sync',   // Ignorovat chyby, chybějící data doplnit nulami
	  errlog: '',             // Soubor pro chybové hlášky
	  hashformat: 'hex',      // Formát výpisu průběžných hashů
	  totalhashformat: 'hex', // Formát výpisu celkového hashe
	  hashconv: 'before',     // Hashovat před konverzemi (typické pro forenzní)
	  diffwr: 'off',          // Vypnuto – neporovnává a nepřepisuje jen změněné bloky
	  sizeprobe: 'if'         // Velikost pro % indikátor – dle vstupního souboru/zařízení
	};
  
	let showKeyboard = false;
	let activeInput: string = '';
  
	async function onFormSubmit(): Promise<void> {
	  try {
		const result = await invoke("save_new_dd_config", formData);
		console.log("Konfigurace DCFLDD byla uložena:", formData);
		dispatch("response", formData);
		dispatch("close");
		showKeyboard = false;
	  } catch (error) {
		console.error("Chyba při odesílání formuláře:", error);
	  }
	}
  
	function onInputFocus(event: Event, fieldName: string) {
	  console.log("Input focused:", fieldName);
	  showKeyboard = true;
	  activeInput = fieldName;
	}
  </script>
  
  <Modal
	{open}
	onOpenChange={(e) => {
	  if (!e.open) {
		dispatch("close");
		showKeyboard = false;
	  }
	}}
	triggerBase="hidden"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-md"
	backdropClasses="backdrop-blur-sm"
  >
	{#snippet trigger()}
	  <span style="display: none;"></span>
	{/snippet}
	{#snippet content()}
	  <div class="modal-form">
		<header class="modal-header">
		  <h2>Nová konfigurace DD</h2>
		</header>
		<form on:submit|preventDefault={onFormSubmit} class="modal-form-content">
		  <label class="label">
			<span>Název konfigurace</span>
			<input
			  class="input"
			  name="confname"
			  type="text"
			  maxlength="12"
			  bind:value={formData.confname}
			  required
			  on:focus={(e) => onInputFocus(e, "confname")}
			/>
		  </label>
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
		  <label class="label">
			<span>Velikost vstupního bloku (ibs)</span>
			<input class="input" name="ibs" type="text" bind:value={formData.ibs} on:focus={(e) => onInputFocus(e, "ibs")} />
		  </label>
		  <label class="label">
			<span>Velikost výstupního bloku (obs)</span>
			<input class="input" name="obs" type="text" bind:value={formData.obs} on:focus={(e) => onInputFocus(e, "obs")} />
		  </label>
		  <label class="label">
			<span>Seek offset</span>
			<input class="input" name="seek" type="text" bind:value={formData.seek} on:focus={(e) => onInputFocus(e, "seek")} />
		  </label>
		  <label class="label">
			<span>Skip offset</span>
			<input class="input" name="skip" type="text" bind:value={formData.skip} on:focus={(e) => onInputFocus(e, "skip")} />
		  </label>
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
			<span>Počet bajtů na hash okno (hashwindow)</span>
			<input class="input" name="hashwindow" type="number" bind:value={formData.hashwindow} on:focus={(e) => onInputFocus(e, "hashwindow")} />
		  </label>
		  <label class="label">
			<span>Log soubor pro hash</span>
			<input class="input" name="hashlog" type="text" bind:value={formData.hashlog} on:focus={(e) => onInputFocus(e, "hashlog")} />
		  </label>
		  <label class="label">
			<span>Zobrazovat status zprávy</span>
			<select class="select" bind:value={formData.status}>
			  <option value="on">Zapnuto</option>
			  <option value="off">Vypnuto</option>
			</select>
		  </label>
		  <label class="label">
			<span>Interval status zpráv (statusinterval)</span>
			<input class="input" name="statusinterval" type="number" bind:value={formData.statusinterval} on:focus={(e) => onInputFocus(e, "statusinterval")} />
		  </label>
		  <label class="label">
			<span>Rozdělení souborů po velikosti (split)</span>
			<select class="select" bind:value={formData.split}>
			  <option value="ask">Zeptat se</option>
			  <option value="disabled">Zakázáno</option>
			</select>
		  </label>
		  <label class="label">
			<span>Formát rozdělení souboru (splitformat)</span>
			<input class="input" name="splitformat" type="text" bind:value={formData.splitformat} on:focus={(e) => onInputFocus(e, "splitformat")} />
		  </label>
		  <label class="label">
			<span>Ověření souboru (vf)</span>
			<input class="input" name="vf" type="text" bind:value={formData.vf} on:focus={(e) => onInputFocus(e, "vf")} />
		  </label>
		  <label class="label">
			<span>Log soubor pro ověření</span>
			<input class="input" name="verifylog" type="text" bind:value={formData.verifylog} on:focus={(e) => onInputFocus(e, "verifylog")} />
		  </label>
		  <label class="label">
			<span>Chování při chybách (conv)</span>
			<input class="input" name="conv" type="text" bind:value={formData.conv} on:focus={(e) => onInputFocus(e, "conv")} />
		  </label>
		  <label class="label">
			<span>Log soubor pro chyby (errlog)</span>
			<input class="input" name="errlog" type="text" bind:value={formData.errlog} on:focus={(e) => onInputFocus(e, "errlog")} />
		  </label>
		  <label class="label">
			<span>Formát průběžného hashe (hashformat)</span>
			<select class="select" bind:value={formData.hashformat}>
			  <option value="hex">hex</option>
			  <option value="base64">base64</option>
			</select>
		  </label>
		  <label class="label">
			<span>Formát celkového hashe (totalhashformat)</span>
			<select class="select" bind:value={formData.totalhashformat}>
			  <option value="hex">hex</option>
			  <option value="base64">base64</option>
			</select>
		  </label>
		  <label class="label">
			<span>Kdy provádět hashování (hashconv)</span>
			<select class="select" bind:value={formData.hashconv}>
			  <option value="before">before</option>
			  <option value="after">after</option>
			</select>
		  </label>
		  <label class="label">
			<span>Porovnání bloků a zápis jen změněných (diffwr)</span>
			<select class="select" bind:value={formData.diffwr}>
			  <option value="off">off</option>
			  <option value="on">on</option>
			</select>
		  </label>
		  <label class="label">
			<span>Velikost pro procentuální indikátor (sizeprobe)</span>
			<select class="select" bind:value={formData.sizeprobe}>
			  <option value="if">if (zdroj)</option>
			  <option value="of">of (výstup)</option>
			</select>
		  </label>
		  <div class="modal-footer">
			<button type="submit" class="btn variant-filled">Uložit</button>
		  </div>
		</form>
		{#if showKeyboard}
		  <VirtualKeyboard bind:formData {activeInput} on:closeKeyboard={() => { console.log("Keyboard closed"); showKeyboard = false; }} />
		{/if}
	  </div>
	{/snippet}
  </Modal>
  
  {#if showKeyboard}
	<!-- Pokud chcete, můžete VirtualKeyboard vykreslit i mimo modal, ale většinou postačí uvnitř -->
	<!-- <VirtualKeyboard bind:formData {activeInput} on:closeKeyboard={() => (showKeyboard = false)} /> -->
  {/if}
  
  <style>
	.modal-form {
	  width: 90vh;
	  height: 90vh;
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
	.modal-header {
	  margin-bottom: 1rem;
	}
	.label {
	  display: flex;
	  flex-direction: column;
	  margin-bottom: 1rem;
	}
	.modal-footer {
	  display: flex;
	  justify-content: flex-end;
	  gap: 1rem;
	}
	.segment-size-inputs {
	  display: flex;
	  gap: 0.5rem;
	}
	.segment-size-inputs .input,
	.segment-size-inputs .select {
	  flex: 1;
	}
  </style>