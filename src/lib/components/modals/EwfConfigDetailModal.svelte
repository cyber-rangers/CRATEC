<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { onMount } from 'svelte';

	export let config: any;
	export let openState: boolean = false;

	function modalClose() {
		openState = false;
	}
	// Po otevření zajistíme, že se obsah modalu (tabulka) posune na začátek
	onMount(() => {
		setTimeout(() => {
			const modalElement = document.querySelector('.table-wrap');
			if (modalElement) {
				modalElement.scrollTop = 0;
			}
		}, 10);
	});
</script>

<Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>

	{#snippet content()}
		<div class="table-wrap">
			<table class="table caption-bottom">
				<thead>
					<tr>
						<th>Položka</th>
						<th>Hodnota</th>
					</tr>
				</thead>
				<tbody>
					<tr>
						<td><strong>Název</strong></td>
						<td>{config?.confname || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Datum vytvoření</strong></td>
						<td>{config?.created || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Kódová stránka</strong></td>
						<td>{config?.codepage || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Počet sektorů na čtení</strong></td>
						<td>{config?.sectors_per_read || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Bajty k získání</strong></td>
						<td>{config?.bytes_to_read || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Metoda komprese</strong></td>
						<td>{config?.compression_method || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Úroveň komprese</strong></td>
						<td>{config?.compression_level || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Formát EWF</strong></td>
						<td>{config?.ewf_format || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Granularita sektorů</strong></td>
						<td>{config?.granularity_sectors || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Hash typy</strong></td>
						<td>{config?.hash_types || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Poznámky</strong></td>
						<td>{config?.notes || 'Bez poznámek'}</td>
					</tr>
					<tr>
						<td><strong>Offset</strong></td>
						<td>{config?.offset || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Velikost bufferu procesu</strong></td>
						<td>{config?.process_buffer_size || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Bajty na sektor</strong></td>
						<td>{config?.bytes_per_sector || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Počet opakování při chybě čtení</strong></td>
						<td>{config?.read_retry_count || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Přehazování bajtových párů</strong></td>
						<td>{config?.swap_byte_pairs ? 'Ano' : 'Ne'}</td>
					</tr>
					<tr>
						<td><strong>Velikost segmentu</strong></td>
						<td>{config?.segment_size || 'Není k dispozici'}</td>
					</tr>
					<tr>
						<td><strong>Nulování při chybě čtení</strong></td>
						<td>{config?.zero_on_read_error ? 'Ano' : 'Ne'}</td>
					</tr>
					<tr>
						<td><strong>Použití chunk dat</strong></td>
						<td>{config?.use_chunk_data ? 'Ano' : 'Ne'}</td>
					</tr>
					
				</tbody>
			</table>
			<div class="modal-footer">
				<button class="btn preset-filled-primary-500" on:click={modalClose}>Zavřít</button>
			</div>
		</div>
	{/snippet}
</Modal>

<style lang="postcss">
	.table-wrap {
		width: 90vh;
		padding: 10px;
		max-height: 90vh;
		overflow-y: auto;
		scroll-behavior: smooth;
	}

	.table-wrap::-webkit-scrollbar {
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

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		margin-top: 1rem;
	}
</style>
