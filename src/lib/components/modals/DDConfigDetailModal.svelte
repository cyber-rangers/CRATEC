<script lang="ts">
    import { Modal } from '@skeletonlabs/skeleton-svelte';
    import { onMount } from 'svelte';

    export let config: any;
    export let openState: boolean = false;

    function modalClose() {
        openState = false;
    }

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
                        <td><strong>Vytvořeno</strong></td>
                        <td>{config?.created || 'Není k dispozici'}</td>
                    </tr>
                    <tr>
                        <td><strong>Aktivní</strong></td>
                        <td>{config?.active ? 'Ano' : 'Ne'}</td>
                    </tr>
                    <tr>
                        <td><strong>Formát</strong></td>
                        <td>{config?.format || 'Není k dispozici'}</td>
                    </tr>
                    <tr>
                        <td><strong>Limit mode</strong></td>
                        <td>{config?.limit_mode || 'Není k dispozici'}</td>
                    </tr>
                    <tr>
                        <td><strong>Offset</strong></td>
                        <td>{config?.offset !== undefined ? config.offset : 'Není k dispozici'}</td>
                    </tr>
                    <tr>
                        <td><strong>Hash typy</strong></td>
                        <td>{config?.hash_types || 'Není k dispozici'}</td>
                    </tr>
                    <tr>
                        <td><strong>Hash window</strong></td>
                        <td>
                            {#if config?.hashwindow && config.hashwindow.toLowerCase() === 'whole'}
                                celý disk
                            {:else}
                                {config?.hashwindow || 'N/A'}
                            {/if}
                        </td>
                    </tr>
                    <tr>
                        <td><strong>Rozdělení</strong></td>
                        <td>
                            {#if config?.split && config.split.toLowerCase() === 'whole'}
                                celý disk
                            {:else}
                                {config?.split || 'N/A'}
                            {/if}
                        </td>
                    </tr>
                    <tr>
                        <td><strong>VF</strong></td>
                        <td>{config?.vf ? 'Zapnuto' : 'Vypnuto'}</td>
                    </tr>
                    <tr>
                        <td><strong>Diffwr</strong></td>
                        <td>{config?.diffwr ? 'Zapnuto' : 'Vypnuto'}</td>
                    </tr>
                    <tr>
                        <td><strong>Poznámky</strong></td>
                        <td>{config?.notes || 'Není k dispozici'}</td>
                    </tr>
                    <tr>
                        <td><strong>ID</strong></td>
                        <td>{config?.id || 'Není k dispozici'}</td>
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
		width: 100%;
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
		width: 80vh;
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

	.table tbody tr:hover {
		background-color: inherit !important;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		margin-top: 1rem;
	}
</style>