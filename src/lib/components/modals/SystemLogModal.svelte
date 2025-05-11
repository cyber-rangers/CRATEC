<script lang="ts">
    import { Modal, Pagination, Popover } from '@skeletonlabs/skeleton-svelte';
    import {
        ArrowLeft as IconArrowLeft,
        ArrowRight as IconArrowRight,
        Ellipsis as IconEllipsis,
        ChevronsLeft as IconFirst,
        ChevronRight as IconLast,
        X as IconX
    } from 'lucide-svelte';
    import { invoke } from '@tauri-apps/api/core';

    export let openState: boolean;

    interface SystemLog {
        id: number;
        level: string;
        message: string;
        timestamp: string;
    }

    let logs: SystemLog[] = [];
    let page = 1;
    let size = 10;
    let popoverOpen: number | null = null;

    // Načti logy při otevření modalu
    $: if (openState) {
        (async () => {
            try {
                logs = await invoke<SystemLog[]>('get_system_logs');
                page = 1; // resetuj stránkování při každém otevření
            } catch (e) {
                console.error('Chyba při načítání logů:', e);
            }
        })();
    }

    function slicedLogs() {
        return logs.slice((page - 1) * size, page * size);
    }

    function modalClose() {
        openState = false;
    }

    function formatShortTime(ts: string): string {
        const d = new Date(ts);
        const pad = (n: number) => n.toString().padStart(2, '0');
        return `${pad(d.getDate())}.${pad(d.getMonth() + 1)}.${d.getFullYear()} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
    }
</script>

<Modal
    open={openState}
    onOpenChange={(e) => (openState = e.open)}
    contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-md"
    backdropClasses="backdrop-blur-sm"
>
    {#snippet content()}
        <header class="mb-2 flex items-center justify-between">
            <h2 class="h3">Systémové Logy</h2>
            <button class="btn-icon" on:click={modalClose} aria-label="Zavřít">
                <IconX />
            </button>
        </header>
        <section class="space-y-4">
            <!-- Table -->
            <div class="table-wrap overflow-x-auto">
                <table class="table table-fixed caption-bottom">
                    <thead>
                        <tr>
                            <th class="w-[40px] px-2 whitespace-nowrap">ID</th>
                            <th class="w-[70px] px-2 whitespace-nowrap">Level</th>
                            <th class="max-w-[500px] px-2">Zpráva</th>
                            <th class="w-[140px] !text-right whitespace-nowrap px-2">Čas</th>
                        </tr>
                    </thead>
                    <tbody class="[&>tr]:hover:preset-tonal-primary">
                        {#each slicedLogs() as row (row.id)}
                            <tr>
                                <td class="w-[40px] px-2 whitespace-nowrap">{row.id}</td>
                                <td class="w-[70px] px-2 whitespace-nowrap">{row.level}</td>
                                <td class="max-w-[500px] px-2 truncate">
                                    <Popover
                                        open={popoverOpen === row.id}
                                        onOpenChange={(e) => (popoverOpen = e.open ? row.id : null)}
                                        positioning={{ placement: 'top' }}
                                        triggerBase="w-full text-left bg-transparent border-0 p-0 m-0 truncate cursor-pointer"
                                        contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[700px]"
                                        arrow
                                        arrowBackground="!bg-surface-200 dark:!bg-surface-800"
                                        zIndex="1000"
                                    >
                                        {#snippet trigger()}
                                            <span>{row.message}</span>
                                        {/snippet}
                                        {#snippet content()}
                                            <header class="flex justify-end">
                                                <button
                                                    class="btn-icon hover:preset-tonal"
                                                    on:click={() => (popoverOpen = null)}><IconX /></button>
                                            </header>
                                            <article>
                                                <p class="break-words opacity-80">{row.message}</p>
                                            </article>
                                        {/snippet}
                                    </Popover>
                                </td>
                                <td class="w-[140px] text-right whitespace-nowrap px-2">{formatShortTime(row.timestamp)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
            <!-- Footer -->
            <footer class="flex justify-end">
                <Pagination
                    data={logs}
                    {page}
                    onPageChange={(e) => (page = e.page)}
                    pageSize={size}
                    onPageSizeChange={(e) => (size = e.pageSize)}
                    siblingCount={1}
                >
                    {#snippet labelEllipsis()}<IconEllipsis class="size-4" />{/snippet}
                    {#snippet labelNext()}<IconArrowRight class="size-4" />{/snippet}
                    {#snippet labelPrevious()}<IconArrowLeft class="size-4" />{/snippet}
                    {#snippet labelFirst()}<IconFirst class="size-4" />{/snippet}
                    {#snippet labelLast()}<IconLast class="size-4" />{/snippet}
                </Pagination>
            </footer>
        </section>
    {/snippet}
</Modal>
