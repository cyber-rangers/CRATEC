<script lang="ts">
    import { Pagination } from '@skeletonlabs/skeleton-svelte';
    // Icons
    import { ArrowLeft, ArrowRight, Ellipsis, ChevronLeft, ChevronRight } from 'lucide-svelte';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    type HistoryItem = {
        process: {
            status: string;
            end_datetime: string;
            start_datetime: string;
            // ... další vlastnosti, pokud jsou potřeba ...
        };
        copy_log: Record<string, any> | null;
    };

    // State pro historická data
    let historyData: HistoryItem[] = $state([]);
    // Paginace stav
    let page = $state(1);
    let size = $state(11);
    const slicedHistory = $derived((data: HistoryItem[]) => data.slice((page - 1) * size, page * size));

    onMount(async () => {
        try {
            const data = await invoke<HistoryItem[]>('get_history');
            console.log('get_history →', data);
            historyData = data;
        } catch (e) {
            console.error('Failed to load history:', e);
        }
    });
</script>

<section class="space-y-4">
    <!-- Tabulka -->
    <div class="table-wrap">
        <table class="table table-fixed caption-bottom">
            <thead>
                <tr>
                    <th>Status</th>
                    <th>End Time</th>
                    <th>Start Time</th>
                </tr>
            </thead>
            <tbody class="[&>tr]:hover:preset-tonal-primary">
                {#each slicedHistory(historyData) as row}
                    <tr>
                        <td>{row.process.status}</td>
                        <td>{row.process.end_datetime}</td>
                        <td>{row.process.start_datetime}</td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
    <!-- Patička s paginací -->
    <footer class="fixed bottom-4 right-4">
        <Pagination
            data={historyData}
            {page}
            onPageChange={(e) => (page = e.page)}
            pageSize={size}
            onPageSizeChange={(e) => (size = e.pageSize)}
            siblingCount={4}
        >
            {#snippet labelEllipsis()}<Ellipsis class="size-4" />{/snippet}
            {#snippet labelNext()}<ArrowRight class="size-4" />{/snippet}
            {#snippet labelPrevious()}<ArrowLeft class="size-4" />{/snippet}
            {#snippet labelFirst()}<ChevronLeft class="size-4" />{/snippet}
            {#snippet labelLast()}<ChevronRight class="size-4" />{/snippet}
        </Pagination>
    </footer>
</section>

