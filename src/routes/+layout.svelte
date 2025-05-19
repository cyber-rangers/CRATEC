<script lang="ts">
	import '../app.css';
	let { children } = $props();

	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	onMount(() => {
		getCurrentWindow().setResizable(false);

		const stopWheel = (e: WheelEvent) => {
			if (e.ctrlKey) e.preventDefault();
		};
		window.addEventListener('wheel', stopWheel, { passive: false });

		const stopGesture = (e: Event) => e.preventDefault();
		['gesturestart', 'gesturechange', 'gestureend'].forEach((type) =>
			window.addEventListener(type, stopGesture, { passive: false })
		);

		const stopTouch = (e: TouchEvent) => {
			if (e.touches.length > 1) e.preventDefault();
		};
		window.addEventListener('touchstart', stopTouch, { passive: false });
		window.addEventListener('touchmove', stopTouch, { passive: false });

		const stopCtx = (e: MouseEvent | TouchEvent) => e.preventDefault();
		window.addEventListener('contextmenu', stopCtx, { passive: false });

		return () => {
			window.removeEventListener('wheel', stopWheel);
			['gesturestart', 'gesturechange', 'gestureend'].forEach((type) =>
				window.removeEventListener(type, stopGesture)
			);
			window.removeEventListener('touchstart', stopTouch);
			window.removeEventListener('touchmove', stopTouch);
			window.removeEventListener('contextmenu', stopCtx);
		};
	});
</script>

{@render children()}

<style>
	html,
	body {
		touch-action: none;
		-webkit-touch-callout: none;
	}
</style>
