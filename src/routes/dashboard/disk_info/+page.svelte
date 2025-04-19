<script lang="ts">
    import { Accordion } from '@skeletonlabs/skeleton-svelte';
    import { HardDrive } from 'lucide-svelte';
    import { deviceStore } from '$lib/stores/deviceStore';
    import { get, writable } from 'svelte/store';
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';

    // Hodnota pro řízení aktuálně otevřených položek v novém Accordion stylu
    let value: string[] = [];

    // Store pro lsblk JSON
    let lsblkDetails = writable<Record<string, any>>({});

    // Spojí usb_devices a sata_devices do jednoho pole
    $: allDisks = [
        ...$deviceStore.usb_devices.map((d) => ({ ...d, type: 'usb' })),
        ...$deviceStore.sata_devices.map((d) => ({ ...d, type: 'sata' }))
    ];
    $: console.log('allDisks:', allDisks);

    // Získá lsblk JSON pro dané zařízení
    async function fetchLsblkForDisk(diskInterface: string) {
        const devnode = `/dev/disk/by-path/${diskInterface}`;
        try {
            const result = await invoke('get_lsblk_json', { device: devnode });
            console.log(`lsblk result for ${devnode}:`, result);
            lsblkDetails.update((details) => ({ ...details, [diskInterface]: result }));
        } catch (e) {
            console.error(`Error fetching lsblk info for ${devnode}`, e);
        }
    }

    // Po načtení zavolá fetch pro každé zařízení
    onMount(() => {
        allDisks.forEach((disk) => {
            if (!get(lsblkDetails)[disk.interface]) {
                fetchLsblkForDisk(disk.interface);
            }
        });
    });

    // Při změně Accordion hodnoty uložíme novou hodnotu
    function handleValueChange(details: { value: string[] }) {
        value = details.value;
    }
</script>

<Accordion {value} onValueChange={handleValueChange} collapsible>
    {#each allDisks as disk, i}
        <Accordion.Item value={disk.name || 'Unknown'}>
            {#snippet lead()}
                <HardDrive size={24} />
            {/snippet}
            {#snippet control()}
                {disk.type.toUpperCase()} {disk.name}
            {/snippet}
            {#snippet panel()}
                {#if $lsblkDetails[disk.interface] &&
                    $lsblkDetails[disk.interface].blockdevices &&
                    $lsblkDetails[disk.interface].blockdevices.length > 0}
                    {@const dev = $lsblkDetails[disk.interface].blockdevices[0]}
                    <div class="device-info">
                        <p><strong>Jméno:</strong> {dev.name}</p>
                        <p><strong>Model:</strong> {dev.model || 'Unknown'}</p>
                        <p><strong>Serial:</strong> {dev.serial || 'Unknown'}</p>
                        <p><strong>Velikost</strong> {dev.size || 'Unknown'}</p>
                        <p><strong>Typ:</strong> {dev.type || 'Unknown'}</p>
                        <p><strong>Mountpoint:</strong> {dev.mountpoint || 'None'}</p>
                        <p><strong>Cesta:</strong> {dev.path || 'Unknown'}</p>
                        <p><strong>Transport:</strong> {dev.tran || 'Unknown'}</p>
                        <p><strong>Výrobce:</strong> {dev.vendor || 'Unknown'}</p>
                        <p><strong>HCTL:</strong> {dev.hctl || 'Unknown'}</p>
                        <p><strong>Disk Seq:</strong> {dev["disk-seq"] || 'Unknown'}</p>
                        <p><strong>Fstype:</strong> {dev.fstype || 'None'}</p>
                        <p><strong>Status:</strong> {dev.state || 'Unknown'}</p>
                        <p><strong>RA:</strong> {dev.ra || 'Unknown'}</p>
                    </div>
                    <br />
                    {#if dev.children && dev.children.length > 0}
                        <h3>Oddíly:</h3>
                        {#each dev.children as part}
                            <div class="partition">
                                <p><strong>Jméno oddílu:</strong> {part.name}</p>
                                <p><strong>Typ:</strong> {part.label || 'None'}</p>
                                <p><strong>Velikost:</strong> {part.size || 'Unknown'}</p>
                                <p><strong>Souborový systém:</strong> {part.fstype || 'Unknown'}</p>
                            </div>
                        {/each}
                    {/if}
                {:else}
                    <p>Loading disk information...</p>
                {/if}
            {/snippet}
        </Accordion.Item>
        {#if i < allDisks.length - 1}
            <hr class="hr" />
        {/if}
    {/each}
</Accordion>

<style lang="postcss">
    .accordion-background {
        background-color: var(--color-surface-900);
        padding: 20px;
        border-radius: 20px;
        margin-bottom: 2rem;
    }
    .hr {
        border: 0;
        border-top: 1px solid var(--color-surface-400);
        margin: 12px 0;
    }
    .device-info p {
        margin: 0.15rem 0;
    }
    .partition {
        margin-bottom: 1rem;
        padding: 0.5rem;
        border: 1px solid #444;
        border-radius: 4px;
    }
    pre {
        background: #333;
        color: #fff;
        padding: 1rem;
        border-radius: 4px;
        overflow: auto;
        margin-top: 1rem;
    }
</style>
