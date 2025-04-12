<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import SimpleKeyboard from 'simple-keyboard';
	import 'simple-keyboard/build/css/index.css';
	import { onDestroy, tick} from 'svelte';

	export let showKeyboard: boolean;
	export let activeInput: string;
	export let formData: Record<string, string | string[]>;

	let keyboardContainer: HTMLDivElement;
	let keyboard: SimpleKeyboard | null = null;

	let posX = window.innerWidth / 2 - (window.innerWidth * 0.7) / 2;
	let posY = window.innerHeight - 300;

	let isDragging = false;
	let dragStartX = 0;
	let dragStartY = 0;
	let origX = posX;
	let origY = posY;

	function onDragStart(e: PointerEvent | TouchEvent) {
		e.preventDefault();
		let clientX: number, clientY: number;
		if (e instanceof TouchEvent) {
			clientX = e.touches[0].clientX;
			clientY = e.touches[0].clientY;
		} else {
			clientX = e.clientX;
			clientY = e.clientY;
		}
		isDragging = true;
		dragStartX = clientX;
		dragStartY = clientY;
		origX = posX;
		origY = posY;
		window.addEventListener('pointermove', onDragMove);
		window.addEventListener('pointerup', onDragEnd);
		window.addEventListener('touchmove', onDragMove, { passive: false });
		window.addEventListener('touchend', onDragEnd);
	}

	function onDragMove(e: PointerEvent | TouchEvent) {
		if (!isDragging) return;
		e.preventDefault();
		let clientX: number, clientY: number;
		if (e instanceof TouchEvent) {
			clientX = e.touches[0].clientX;
			clientY = e.touches[0].clientY;
		} else {
			clientX = e.clientX;
			clientY = e.clientY;
		}
		posX = origX + (clientX - dragStartX);
		posY = origY + (clientY - dragStartY);
	}

	function onDragEnd() {
		isDragging = false;
		window.removeEventListener('pointermove', onDragMove);
		window.removeEventListener('pointerup', onDragEnd);
		window.removeEventListener('touchmove', onDragMove);
		window.removeEventListener('touchend', onDragEnd);
	}

	function handleKeyPress(button: string) {
		if (!activeInput) return;
		const currentValue = formData[activeInput];

		if (typeof currentValue !== 'string') return;

		if (button === '{shift}') {
			if (keyboard) {
				const currentLayout = keyboard.options.layoutName;
				const newLayout = currentLayout === 'default' ? 'shift' : 'default';
				keyboard.setOptions({ layoutName: newLayout });
			}
			return;
		} else if (button === '{bksp}') {
			formData[activeInput] = currentValue.slice(0, -1);
		} else if (button === '{space}') {
			formData[activeInput] = currentValue + ' ';
		} else {
			formData[activeInput] = currentValue + button;
		}

		const inputEl = document.querySelector(`[name="${activeInput}"]`) as HTMLInputElement;
		if (inputEl) {
			inputEl.value = formData[activeInput] as string;
			inputEl.focus();
			const end = inputEl.value.length;
			inputEl.setSelectionRange(end, end);
		}
	}

	async function initKeyboard() {
		await tick();
		if (keyboardContainer && !keyboard) {
			keyboard = new SimpleKeyboard(keyboardContainer, {
				onKeyPress: handleKeyPress,
				physicalKeyboardHighlight: true,
				theme: 'hg-theme-default cratec-theme',

				// DŮLEŽITÉ: vynutíme zpracování dotykových i myš událostí
				useTouchEvents: true,
				useMouseEvents: true,

				layout: {
					default: [
						'ě š č ř ž ý á í é -',
						'q w e r t y u i o p {bksp}',
						'a s d f g h j k l',
						'{shift} z x c v b n m .',
						'{space}'
					],
					shift: [
						'1 2 3 4 5 6 7 8 9 0',
						'Q W E R T Y U I O P {bksp}',
						'A S D F G H J K L',
						'{shift} Z X C V B N M',
						'{space}'
					]
				},
				display: {
					'{bksp}': '←',
					'{space}': 'Space'
				},
				maxLength: 12
			});
		}
	}

	$: if (showKeyboard && !keyboard) {
		initKeyboard();
	}

	$: if (!showKeyboard && keyboard) {
		keyboard.destroy();
		keyboard = null;
	}

	onDestroy(() => {
		if (keyboard) {
			keyboard.destroy();
			keyboard = null;
		}
	});

	function closeKeyboard() {
		showKeyboard = false;
	}
</script>

<Modal
	open={showKeyboard}
	onOpenChange={(e) => !e.open && closeKeyboard()}
	modal={false}
	backdropBase=""
	backdropClasses="bg-transparent pointer-events-none"
	positionerClasses="pointer-events-none"
	contentClasses="bg-transparent shadow-none border-none pointer-events-auto"
	preventScroll={false}
	trapFocus={false}
	closeOnInteractOutside={false}
>
	{#snippet content()}
		<div class="absolute z-50" style="top: {posY}px; left: {posX}px;">
			<div class="keyboard-container">
				<div bind:this={keyboardContainer} class="simple-keyboard"></div>
			</div>
			<div class="action-buttons">
				<!-- Tlačítko pro tažení (drag) klávesnice -->
				<button
					class="drag-button btn preset-filled"
					on:pointerdown={onDragStart}
					on:touchstart={onDragStart}
				>
					⇕
				</button>
				<!-- Tlačítko pro zavření klávesnice -->
				<button class="close-button btn preset-filled" on:click={closeKeyboard}>✖</button>
			</div>
		</div>
	{/snippet}
</Modal>

<style>
	:global(.simple-keyboard) {
		width: 70vw;
		max-width: 70vw;
		background: var(--color-surface-800);
		pointer-events: auto;
		touch-action: none;
	}

	.keyboard-container {
		width: 70vw;
		max-width: 70vw;
		box-sizing: border-box;
	}

	.action-buttons {
		position: absolute;
		top: -45px;
		right: -45px;
		width: 80px;
		height: 75px;
	}

	.drag-button,
	.close-button {
		background: var(--color-surface-600);
		color: #fff;
		border-radius: 50%;
		width: 40px;
		height: 40px;
		font-size: 1.2rem;
		display: flex;
		align-items: center;
		justify-content: center;
		position: absolute;
		cursor: pointer;
	}

	/* Pro drag tlačítko je také vhodné nastavit touch-action: none */
	.drag-button {
		top: 0;
		left: 0;
		touch-action: none;
	}
	.close-button {
		top: 30px;
		right: 0;
	}

	:global(.simple-keyboard .hg-button) {
		height: 40px;
		margin: 2px;
		background: var(--color-surface-600);
		color: white;
		font-weight: 900;
		border-radius: 4px;
	}
</style>
