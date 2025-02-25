<script lang="ts">
	import { getModalStore } from "@skeletonlabs/skeleton";
	import { onMount } from "svelte";

	const modalStore = getModalStore();

	// Načtení dat konfigurace z meta
	$: modalData = $modalStore.length ? $modalStore[0].meta.configData : {};

	// Ladicí výpis do konzole
	$: console.log("Detail modal data:", modalData);

	// Zavření modalu
	function closeModal() {
		modalStore.close();
	}

	// Funkce pro posunutí na začátek při otevření
	onMount(() => {
		setTimeout(() => {
			const modalElement = document.querySelector(".table-container");
			if (modalElement) {
				modalElement.scrollTop = 0;
			}
		}, 10); // Malé zpoždění pro zajištění správného posunu
	});
</script>

<div class="table-container">
	<!-- Tabulka s konfigurací -->
	<table class="table table-compact">
		<thead>
			<tr>
				<th>Položka</th>
				<th>Hodnota</th>
			</tr>
		</thead>
		<tbody>
			<tr><td><strong>Název</strong></td><td>{modalData?.confname || "Není k dispozici"}</td></tr>
			<tr><td><strong>Kódová stránka</strong></td><td>{modalData?.codepage || "Není k dispozici"}</td></tr>
			<tr><td><strong>Počet sektorů na čtení</strong></td><td>{modalData?.sectors_per_read || "Není k dispozici"}</td></tr>
			<tr><td><strong>Bajty k získání</strong></td><td>{modalData?.bytes_to_read || "Není k dispozici"}</td></tr>
			<tr><td><strong>Metoda komprese</strong></td><td>{modalData?.compression_method || "Není k dispozici"}</td></tr>
			<tr><td><strong>Úroveň komprese</strong></td><td>{modalData?.compression_level || "Není k dispozici"}</td></tr>
			<tr><td><strong>Formát EWF</strong></td><td>{modalData?.ewf_format || "Není k dispozici"}</td></tr>
			<tr><td><strong>Granularita sektorů</strong></td><td>{modalData?.granularity_sectors || "Není k dispozici"}</td></tr>
			<tr><td><strong>Hash typy</strong></td><td>{modalData?.hash_types || "Není k dispozici"}</td></tr>
			<tr><td><strong>Poznámky</strong></td><td>{modalData?.notes || "Bez poznámek"}</td></tr>
			<tr><td><strong>Offset</strong></td><td>{modalData?.offset || "Není k dispozici"}</td></tr>
			<tr><td><strong>Velikost bufferu procesu</strong></td><td>{modalData?.process_buffer_size || "Není k dispozici"}</td></tr>
			<tr><td><strong>Bajty na sektor</strong></td><td>{modalData?.bytes_per_sector || "Není k dispozici"}</td></tr>
			<tr><td><strong>Tichý režim</strong></td><td>{modalData?.quiet_mode ? "Ano" : "Ne"}</td></tr>
			<tr><td><strong>Počet opakování při chybě čtení</strong></td><td>{modalData?.read_retry_count || "Není k dispozici"}</td></tr>
			<tr><td><strong>Přehazování bajtových párů</strong></td><td>{modalData?.swap_byte_pairs ? "Ano" : "Ne"}</td></tr>
			<tr><td><strong>Velikost segmentu</strong></td><td>{modalData?.segment_size || "Není k dispozici"}</td></tr>
			<tr><td><strong>Podrobný výstup</strong></td><td>{modalData?.verbose_output ? "Ano" : "Ne"}</td></tr>
			<tr><td><strong>Nulování při chybě čtení</strong></td><td>{modalData?.zero_on_read_error ? "Ano" : "Ne"}</td></tr>
			<tr><td><strong>Použití chunk dat</strong></td><td>{modalData?.use_chunk_data ? "Ano" : "Ne"}</td></tr>
			<tr><td><strong>Datum vytvoření</strong></td><td>{modalData?.created || "Není k dispozici"}</td></tr>
			<tr><td><strong>Aktivní</strong></td><td>{modalData?.active ? "Ano" : "Ne"}</td></tr>
		</tbody>
	</table>

	<!-- Ovládací tlačítko pouze pro zavření -->
	<div class="modal-footer">
		<button class="btn btn-secondary" on:click={closeModal}>Zavřít</button>
	</div>
</div>

<style lang="postcss">
	.table-container {
		width: 60%;
		padding: 25px;
		max-height: 90vh;
		overflow-y: auto;
		scroll-behavior: smooth; /* Přidává plynulý efekt posunu */
	}

	.table-container::-webkit-scrollbar {
		width: 1px;
		background-color: transparent;
	}

	.table {
		width: 100%;
		border-collapse: collapse;
	}

	.table th,
	.table td {
		border: 1px solid #ddd;
		padding: 8px;
		text-align: left;
	}

	.table th {
		background-color: rgba(var(--color-surface-600) / 1);
	}

	/* Odstranění hover efektu na řádcích */
	.table tbody tr:hover {
		background-color: inherit !important;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		margin-top: 1rem;
	}
</style>
