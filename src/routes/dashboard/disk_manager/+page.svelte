<script lang="ts">
    import { onMount } from 'svelte';
    import { writable, get } from 'svelte/store';
    import { Usb, HardDrive, File as FileIcon, Folder as FolderIcon, ChevronLeft } from 'lucide-svelte';
    import { deviceStore } from '$lib/stores/deviceStore';
    import { invoke } from '@tauri-apps/api/core';
    import type { DeviceBase } from '$lib/stores/deviceStore';

    interface FileItem {
        name: string;
        file_type: string;
        size: number;
        created: number | null;
    }

    const selectedDisk = writable<any>(null);
    const fileItems = writable<FileItem[]>([]);
    const pathStack = writable<string[]>([]);

    let disks: DeviceBase[] = [];

    const unsubscribe = deviceStore.subscribe((value) => {
        disks = [...value.usb_devices, ...value.sata_devices].filter((d) => d.mountpoint);
    });

    async function selectDisk(disk: any) {
        selectedDisk.set(disk);
        pathStack.set([disk.mountpoint]);
        await loadDirectory(disk.mountpoint);
    }

    async function loadDirectory(path: string) {
        try {
            const result = await invoke<FileItem[]>('get_directory_contents', { mountpoint: path });
            fileItems.set(result);
        } catch (err) {
            console.error('Failed to get directory contents:', err);
            fileItems.set([]);
        }
    }

    function goBack() {
        const stack = get(pathStack);
        if (stack.length > 1) {
            stack.pop();
            pathStack.set([...stack]);
            loadDirectory(stack[stack.length - 1]);
        } else {
            selectedDisk.set(null);
            fileItems.set([]);
            pathStack.set([]);
        }
    }

    function openFolder(folderName: string) {
        const stack = get(pathStack);
        const newPath = stack[stack.length - 1] + '/' + folderName;
        pathStack.set([...stack, newPath]);
        loadDirectory(newPath);
    }

    function formatSize(size: number) {
        if (size === 0) return '';
        if (size < 1024) return `${size} B`;
        if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
        if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`;
        return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    }

    function formatDate(ts: number | null) {
        if (!ts) return '';
        const d = new Date(ts * 1000);
        return d.toLocaleString();
    }
</script>

{#if $selectedDisk === null}
    <!-- Zobrazení dostupných disků -->
    <div class="disk-container">
        {#each disks as disk (disk.interface)}
            <div class="disk-box" on:click={() => selectDisk(disk)}>
                <div class="icon">
                    {#if disk.type === 'usb'}
                        <Usb size="48" />
                    {:else if disk.type === 'sata'}
                        <HardDrive size="48" />
                    {/if}
                </div>
                <div class="disk-info">
                    <div class="disk-name">{disk.name || disk.interface}</div>
                    <div class="mountpoint">{disk.mountpoint}</div>
                </div>
            </div>
        {/each}
    </div>
{:else}
    <!-- Zobrazení obsahu vybraného disku v tabulce -->
    <div class="filemanager-view">
        <div class="table-wrap">
            <table class="table caption-bottom">
                <caption class="pt-4">Obsah vybrané složky</caption>
                <thead>
                    <tr>
                        <th class="center-th">
                            <button on:click={goBack} title="Zpět" class="back-btn">
                                <ChevronLeft size="28" />
                            </button>
                        </th>
                        <th>Název</th>
                        <th>Typ</th>
                        <th>Velikost</th>
                        <th>Vytvořeno</th>
                    </tr>
                </thead>
                <tbody class="[&>tr]:hover:preset-tonal-primary">
                    {#each $fileItems as item}
                        <tr>
                            <td>
                                {#if item.file_type === 'folder'}
                                    <FolderIcon size="20" />
                                {:else}
                                    <FileIcon size="20" />
                                {/if}
                            </td>
                            <td>
                                {#if item.file_type === 'folder'}
                                    <a href="#" on:click|preventDefault={() => openFolder(item.name)} style="font-weight:bold;">
                                        {item.name}
                                    </a>
                                {:else}
                                    {item.name}
                                {/if}
                            </td>
                            <td>{item.file_type === 'folder' ? 'Složka' : 'Soubor'}</td>
                            <td>{formatSize(item.size)}</td>
                            <td>{formatDate(item.created)}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
{/if}

<style>
    .disk-container {
        display: flex;
        flex-wrap: wrap;
        gap: 1rem;
        padding: 1rem;
    }
    .disk-box {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        background: var(--color-surface-900);
        border: 3px solid var(--color-primary-600);
        border-radius: 8px;
        padding: 1rem;
        cursor: pointer;
        width: 150px;
        height: 120px;
        text-align: center;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        transition: transform 0.2s;
    }
    .disk-box:hover {
        transform: translateY(-5px);
    }
    .disk-info {
        margin-top: 0.5rem;
    }
    .disk-name {
        font-weight: bold;
        margin-bottom: 0.5rem;
    }
    .filemanager-view {
        padding: 1rem;
    }
    .filemanager-view button {
        margin-bottom: 1rem;
    }
    .table-wrap {
        overflow-x: auto;
    }
    table {
        width: 100%;
        border-collapse: collapse;
    }
    th, td {
        padding: 0.5rem 1rem;
        border-bottom: 1px solid #eee;
    }
    th {
        background: #f8f8f8;
        text-align: left;
    }
    th:first-child, td:first-child {
        text-align: center;
        vertical-align: middle !important;
        width: 40px;
        height: 56px; /* výška řádku tabulky, uprav dle potřeby */
        padding: 0;
    }
    tr.folder td {
        font-weight: bold;
    }
    a {
        color: var(--color-primary-600);
        text-decoration: none;
        cursor: pointer;
    }
    a:hover {
        text-decoration: underline;
    }
    .back-btn {
        background: none;
        border: none;
        cursor: pointer;
        color: var(--color-primary-600);
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        height: 40px;
        width: 40px;
    }
    .back-btn:hover {
        color: var(--color-primary-800);
        text-decoration: underline;
    }
    .center-th {
        width: 48px;
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        height: 56px;
    }
    th.center-th {
        width: 56px;
        padding: 0;
        height: 56px;
    }
    th.center-th > .back-btn {
        height: 56px;
        width: 56px;
        display: flex;
        align-items: center;
        justify-content: center;
        margin: 0 auto;
        background: none;
        border: none;
        cursor: pointer;
        color: var(--color-primary-600);
        padding: 0;
    }
    th.center-th > .back-btn:hover {
        color: var(--color-primary-800);
        text-decoration: underline;
    }
</style>
