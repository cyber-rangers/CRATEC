<script>
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { ProgressRadial } from '@skeletonlabs/skeleton';
  // Perform the redirection to /boot when the component is mounted
  onMount(() => {
      setTimeout(() => {
          //goto('/initial_setup');
          goto('/dashboard');
      }, 3000); // Delay in milliseconds (9000ms = 9 seconds)
  });
</script>

<div class="h-screen flex flex-col items-center justify-center relative">
  <img src="/rangers-logo.png" alt="Rangers Logo" class="logo fade-text" />
  <h2 class="fade-text">
      <span class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone">
          CRATEC
      </span>
  </h2>

  
</div>

<style lang="postcss">
  @keyframes fade {
      0% {
          opacity: 1;
      }
      50% {
          opacity: 0.30;
      }
      100% {
          opacity: 1;
      }
  }

  .h-screen {
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;
  }

  h2 {
      font-weight: bold;
      font-size: 5rem;
      margin-bottom: 75px;
  }

  .logo {
      width: 350px;
      height: auto;
  }

  .fade-text {
      animation: fade 2s infinite;
  }
</style>