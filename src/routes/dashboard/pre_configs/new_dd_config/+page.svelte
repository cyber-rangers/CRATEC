<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation'; // Přidáno pro přesměrování
	import VirtualKeyboard from '$lib/components/VirtualKeyboard.svelte';

	let formData = {
		confname: '', // Název konfigurace
		bs: '32768', // Výchozí velikost bloku (32 KB)
		count: 'whole', // Počet bloků k získání
		ibs: '32768', // Velikost vstupního bloku
		obs: '32768', // Velikost výstupního bloku
		seek: '0', // Seek offset na začátku výstupu
		skip: '0', // Skip offset na začátku vstupu
		hash_types: ['md5'], // Výchozí hash (možnost více hashů)
		hashwindow: '4096', // Počet bajtů na hash okno
		hashlog: '', // Log soubor pro hash
		status: 'on', // Zobrazovat status zprávy
		statusinterval: '256', // Interval status zpráv
		split: 'ask', // Rozdělit soubory po určité velikosti
		splitformat: 'nnn', // Formát rozdělení souboru
		vf: 'ask', // Ověření souboru
		verifylog: '', // Log soubor pro ověření

		// Nové parametry:
		conv: 'noerror,sync', // Ignorovat chyby, chybějící data doplnit nulami
		errlog: '', // Soubor pro chybové hlášky
		hashformat: 'hex', // Formát průběžného hashe
		totalhashformat: 'hex', // Formát celkového hashe
		hashconv: 'before', // Hashovat před konverzemi
		diffwr: 'off', // Neporovnává a nepřepisuje jen změněné bloky
		sizeprobe: 'if' // Velikost pro % indikátor – dle vstupního souboru/zařízení
	};

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

	async function onFormSubmit(): Promise<void> {
		try {
			// Uložíme novou konfiguraci vyvoláním rust funkce:
			const result = await invoke('save_new_dd_config', formData);
			console.log('Konfigurace DCFLDD byla uložena:', formData);

			// Po úspěšném uložení přesměrujeme uživatele na přehled konfigurací
			goto('/dashboard/pre_configs');
		} catch (error) {
			console.error('Chyba při odesílání formuláře:', error);
			// Můžete např. zobrazit toast, chybovou hlášku atd.
		}
	}
</script>

<main class="page-wrapper">
	<form class="modal-form" on:submit|preventDefault={onFormSubmit}>
		<label class="label">
			<span>Název konfigurace</span>
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
			<input class="input" name="ibs" type="text" bind:value={formData.ibs} />
		</label>

		<label class="label">
			<span>Velikost výstupního bloku (obs)</span>
			<input class="input" name="obs" type="text" bind:value={formData.obs} />
		</label>

		<label class="label">
			<span>Seek offset</span>
			<input class="input" name="seek" type="text" bind:value={formData.seek} />
		</label>

		<label class="label">
			<span>Skip offset</span>
			<input class="input" name="skip" type="text" bind:value={formData.skip} />
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
			<input class="input" name="hashwindow" type="number" bind:value={formData.hashwindow} />
		</label>

		<label class="label">
			<span>Log soubor pro hash</span>
			<input class="input" name="hashlog" type="text" bind:value={formData.hashlog} />
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
			<input
				class="input"
				name="statusinterval"
				type="number"
				bind:value={formData.statusinterval}
			/>
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
			<input class="input" name="splitformat" type="text" bind:value={formData.splitformat} />
		</label>

		<label class="label">
			<span>Ověření souboru (vf)</span>
			<input class="input" name="vf" type="text" bind:value={formData.vf} />
		</label>

		<label class="label">
			<span>Log soubor pro ověření</span>
			<input class="input" name="verifylog" type="text" bind:value={formData.verifylog} />
		</label>

		<!-- Nové parametry: conv, errlog, hashformat, totalhashformat, hashconv, diffwr, sizeprobe -->
		<label class="label">
			<span>Chování při chybách (conv)</span>
			<input class="input" name="conv" type="text" bind:value={formData.conv} />
		</label>

		<label class="label">
			<span>Log soubor pro chyby (errlog)</span>
			<input class="input" name="errlog" type="text" bind:value={formData.errlog} />
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
