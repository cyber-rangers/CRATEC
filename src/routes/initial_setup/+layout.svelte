<script lang="ts">
    import { fly } from 'svelte/transition'; 
    import { LightSwitch } from '@skeletonlabs/skeleton';
    export let data;
</script>

<div class="absolute top-4 right-4 z-20">
    <LightSwitch width="w-16" height="h-8" />
</div>

<div class="flex h-screen relative">
    
    {#key data.currentPath} 
        <div class="content absolute inset-0 flex-grow p-6 z-10" in:fly={{ duration: 2000, delay: 0, x: -2000 }} out:fly={{ duration: 2000, x: 2000 }}>
            <slot />
        </div>
    {/key}
</div>

<style lang="postcss">
    
    .content {
        display: flex;
        justify-content: center;
        align-items: center;
        position: absolute;
        inset: 0;
    }
</style>
