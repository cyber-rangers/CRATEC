<script lang="ts">
    import { Modal } from '@skeletonlabs/skeleton-svelte';
    
    // Exportujeme stav modalu, aby bylo možné jej ovládat z rodičovské stránky
    export let openState: boolean = false;
  
    function modalClose() {
      openState = false;
    }
  </script>
  
  <Modal
    open={openState}
    onOpenChange={(e) => (openState = e.open)}
    triggerBase="btn preset-tonal"
    contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
    backdropClasses="backdrop-blur-sm"
  >
    {#snippet content()}
      <header class="flex justify-between">
        <h2 class="h2">Modal Example</h2>
      </header>
      <article>
        <p class="opacity-60">
          Lorem ipsum dolor sit amet consectetur adipisicing elit. Nam, ab adipisci. Libero cumque sunt quis error veritatis amet, expedita
          voluptatem. Quos repudiandae consequuntur voluptatem et dicta quas, reprehenderit velit excepturi?
        </p>
      </article>
      <footer class="flex justify-end gap-4">
        <button type="button" class="btn preset-tonal" on:click={modalClose}>Cancel</button>
        <button type="button" class="btn preset-filled" on:click={modalClose}>Confirm</button>
      </footer>
    {/snippet}
  </Modal>
  