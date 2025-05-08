<script lang="ts">
    import { Pagination } from '@skeletonlabs/skeleton-svelte';
    import { ArrowLeft, ArrowRight, Ellipsis, ChevronLeft, ChevronRight } from 'lucide-svelte';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    type HistoryItem = {
        process: {
            status: string;
            end_datetime: string;
            start_datetime: string;
            id?: number; // opraveno na number
            process_id?: number;
        };
        copy_log: Record<string, any> | null;
    };

    let historyData: HistoryItem[] = [];
    let page = 1;
    let size = 11;
    let selectedDetail: HistoryItem | null = null;
    let tab: 'detail' | 'log' = 'detail';
    let processLog: string[] = [];
    let loadingLog = false;

    function slicedHistory(data: HistoryItem[]) {
        return data.slice((page - 1) * size, page * size);
    }

    onMount(async () => {
        try {
            const data = await invoke<HistoryItem[]>('get_history');
            historyData = data;
        } catch (e) {
            console.error('Failed to load history:', e);
        }
    });

    function showDetail(item: HistoryItem) {
        selectedDetail = item;
    }
    function backToTable() {
        selectedDetail = null;
    }

    async function fetchProcessLog() {
        if (!selectedDetail) return;
        loadingLog = true;
        const processId = Number(selectedDetail.process.id ?? selectedDetail.process.process_id);
        try {
            if (!processId) {
                processLog = ['ID procesu nenalezeno.'];
                loadingLog = false;
                return;
            }
            const lines = await invoke<string[]>('get_process_log_lines_texts', { process_id: processId });
            processLog = lines;
        } catch (e) {
            console.error('fetchProcessLog error:', e);
            processLog = ['Chyba při načítání logu.'];
        }
        loadingLog = false;
    }

    $: if (tab === 'log' && selectedDetail) {
        fetchProcessLog();
    }

    // Výpočet doby trvání v čitelném formátu
    function getDuration(start: string, end: string): string {
        if (!start || !end) return '';
        const startDate = new Date(start.replace(' ', 'T'));
        const endDate = new Date(end.replace(' ', 'T'));
        const diffMs = endDate.getTime() - startDate.getTime();
        if (isNaN(diffMs) || diffMs < 0) return '';
        const sec = Math.floor(diffMs / 1000) % 60;
        const min = Math.floor(diffMs / 60000) % 60;
        const hrs = Math.floor(diffMs / 3600000);
        return `${hrs}h ${min}m ${sec}s`;
    }

    // Přidej reaktivní proměnnou pro kontrolu hashů
    $: hasAnyHash =
        selectedDetail?.copy_log?.md5_hash ||
        selectedDetail?.copy_log?.sha1_hash ||
        selectedDetail?.copy_log?.sha256_hash ||
        selectedDetail?.copy_log?.sha384_hash ||
        selectedDetail?.copy_log?.sha512_hash;
</script>

<style>
body, .page-wrapper {
    overflow-x: hidden;
    max-width: 100vw;
}
</style>

{#if !selectedDetail}
<section class="space-y-4">
    <div class="table-wrap">
        <table class="table table-fixed caption-bottom">
            <thead>
                <tr>
                    <th>Status</th>
                    <th>Spuštění akvizice</th>
                    <th>Ukončení akvizice</th>
                </tr>
            </thead>
            <tbody class="[&>tr]:hover:preset-tonal-primary">
                {#each slicedHistory(historyData) as row}
                    <tr onclick={() => showDetail(row)} style="cursor:pointer">
                        <td>{row.process.status}</td>
                        <td>{row.process.end_datetime}</td>
                        <td>{row.process.start_datetime}</td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
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
{:else}
<div class="p-4 space-y-6 overflow-x-hidden">
    <div class="flex items-center justify-between mb-6">
        <button class="btn btn-sm flex items-center gap-2" onclick={backToTable}>
            <ArrowLeft /> Zpět
        </button>
        <nav class="flex-1 flex justify-center">
            <div class="btn-group preset-outlined-surface-200-800">
                <button
                    type="button"
                    class="btn"
                    class:preset-filled={tab === 'detail'}
                    class:hover\:preset-tonal={tab !== 'detail'}
                    aria-pressed={tab === 'detail'}
                    onclick={() => tab = 'detail'}
                >Detail</button>
                <button
                    type="button"
                    class="btn"
                    class:preset-filled={tab === 'log'}
                    class:hover\:preset-tonal={tab !== 'log'}
                    aria-pressed={tab === 'log'}
                    onclick={() => tab = 'log'}
                >Log</button>
            </div>
        </nav>
        <!-- prázdný div pro zarovnání do středu -->
        <div class="w-24"></div>
    </div>

    {#if tab === 'detail'}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
            {#if selectedDetail.copy_log}
            <div>
                <h3 class="h3 mb-2">Kopírovací log</h3>
                <dl class="space-y-2">
                    <div>
                        <dt class="font-bold">Case number</dt>
                        <dd class="opacity-60">{selectedDetail.copy_log.case_number}</dd>
                    </div>
                    <div>
                        <dt class="font-bold">Evidence number</dt>
                        <dd class="opacity-60">{selectedDetail.copy_log.evidence_number}</dd>
                    </div>
                    <div>
                        <dt class="font-bold">Investigator</dt>
                        <dd class="opacity-60">{selectedDetail.copy_log.investigator_name}</dd>
                    </div>
                    <div>
                        <dt class="font-bold">Poznámky</dt>
                        <dd class="opacity-60">{selectedDetail.copy_log.notes}</dd>
                    </div>
                    <div>
                        <dt class="font-bold">Popis</dt>
                        <dd class="opacity-60">{selectedDetail.copy_log.description}</dd>
                    </div>
                </dl>
            </div>
            {/if}
            <div>
                <h2 class="h2 mb-4">Detail procesu</h2>
                <dl class="space-y-2">
                    <div>
                        <dt class="font-bold">Status</dt>
                        <dd class="opacity-60">{selectedDetail.process.status}</dd>
                    </div>
                    <div>
                        <dt class="font-bold">Začátek</dt>
                        <dd class="opacity-60">{selectedDetail.process.start_datetime}</dd>
                    </div>
                    <div>
                        <dt class="font-bold">Konec</dt>
                        <dd class="opacity-60">{selectedDetail.process.end_datetime}</dd>
                    </div>
                    <div>
                        <dt class="font-bold">Doba trvání</dt>
                        <dd class="opacity-60">
                            {getDuration(selectedDetail.process.start_datetime, selectedDetail.process.end_datetime)}
                        </dd>
                    </div>
                </dl>
            </div>
        </div>
        {#if selectedDetail.copy_log && hasAnyHash}
        <div>
            <h3 class="h3 mt-6 mb-2">Hash hodnoty</h3>
            <dl class="space-y-2">
                {#if selectedDetail.copy_log.md5_hash}
                <div>
                    <dt class="font-bold">MD5</dt>
                    <dd class="opacity-60 break-all">
                        <code class="code block break-all whitespace-pre-line">{selectedDetail.copy_log.md5_hash}</code>
                    </dd>
                </div>
                {/if}
                {#if selectedDetail.copy_log.sha1_hash}
                <div>
                    <dt class="font-bold">SHA1</dt>
                    <dd class="opacity-60 break-all">
                        <code class="code block break-all whitespace-pre-line">{selectedDetail.copy_log.sha1_hash}</code>
                    </dd>
                </div>
                {/if}
                {#if selectedDetail.copy_log.sha256_hash}
                <div>
                    <dt class="font-bold">SHA256</dt>
                    <dd class="opacity-60 break-all">
                        <code class="code block break-all whitespace-pre-line">{selectedDetail.copy_log.sha256_hash}</code>
                    </dd>
                </div>
                {/if}
                {#if selectedDetail.copy_log.sha384_hash}
                <div>
                    <dt class="font-bold">SHA384</dt>
                    <dd class="opacity-60 break-all">
                        <code class="code block break-all whitespace-pre-line">{selectedDetail.copy_log.sha384_hash}</code>
                    </dd>
                </div>
                {/if}
                {#if selectedDetail.copy_log.sha512_hash}
                <div>
                    <dt class="font-bold">SHA512</dt>
                    <dd class="opacity-60 break-all">
                        <code class="code block break-all whitespace-pre-line">{selectedDetail.copy_log.sha512_hash}</code>
                    </dd>
                </div>
                {/if}
            </dl>
        </div>
        {/if}
    {:else}
        {#if loadingLog}
            <pre class="pre">Načítám log...</pre>
        {:else if processLog.length > 0}
            <pre class="pre">{processLog.join('\n')}</pre>
        {:else}
            <pre class="pre">Žádné logy k tomuto procesu.</pre>
        {/if}
    {/if}
</div>
{/if}

