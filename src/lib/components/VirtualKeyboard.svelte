<script lang="ts">
	import SimpleKeyboard from "simple-keyboard";
	import "simple-keyboard/build/css/index.css";
	import { onMount, onDestroy, createEventDispatcher, tick } from 'svelte';
  
	export let showKeyboard: boolean;
	export let activeInput: string;
	export let formData: Record<string, string>;
  
	const dispatch = createEventDispatcher();
  
	let keyboardContainer: HTMLDivElement;
	let keyboard: SimpleKeyboard;
  
	// Počáteční pozice klávesnice
	let posX = window.innerWidth / 2 - (window.innerWidth * 0.7) / 2;
	let posY = window.innerHeight - 300;
  
	let isDragging = false;
	let dragStartX = 0, dragStartY = 0;
	let origX = posX, origY = posY;
  
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
	  window.addEventListener("pointermove", onDragMove);
	  window.addEventListener("pointerup", onDragEnd);
	  window.addEventListener("touchmove", onDragMove, { passive: false });
	  window.addEventListener("touchend", onDragEnd);
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
  
	function onDragEnd(e: PointerEvent | TouchEvent) {
	  isDragging = false;
	  window.removeEventListener("pointermove", onDragMove);
	  window.removeEventListener("pointerup", onDragEnd);
	  window.removeEventListener("touchmove", onDragMove);
	  window.removeEventListener("touchend", onDragEnd);
	}
  
	function handleKeyPress(button: string) {
	  if (!activeInput) return;
  
	  const inputEl = document.querySelector(`[name="${activeInput}"]`) as HTMLInputElement;
	  const currentValue = formData[activeInput] || "";
  
	  if (button === "{shift}") {
		const currentLayout = keyboard.options.layoutName;
		const newLayout = currentLayout === "default" ? "shift" : "default";
		keyboard.setOptions({ layoutName: newLayout });
		return;
	  }
  
	  if (button === "{bksp}") {
		formData[activeInput] = String(currentValue).slice(0, -1);
	  } else if (button === "{space}") {
		formData[activeInput] += " ";
	  } else {
		formData[activeInput] += button;
	  }
  
	  // Znovu naplníme <input> reálnou hodnotou
	  if (inputEl) {
		inputEl.value = formData[activeInput];
		inputEl.focus();
		const end = inputEl.value.length;
		inputEl.setSelectionRange(end, end);
	  }
	}
  
	function addTouchListenersToKeys() {
	  const buttons = keyboardContainer.querySelectorAll(".hg-button");
	  buttons.forEach((btn: HTMLElement) => {
		btn.style.touchAction = "manipulation";
		btn.addEventListener("touchstart", (e) => {
		  e.preventDefault();
		});
		btn.addEventListener("touchend", (e) => {
		  e.preventDefault();
		  const btnText = btn.getAttribute("data-skbtn") || btn.innerText;
		  let key = btnText.trim();
		  if (key === "⌫") key = "{bksp}";
		  if (key === "Space") key = "{space}";
		  handleKeyPress(key);
		});
	  });
	}
  
	onMount(async () => {
	  // Zde nic moc, klávesnici teprve tvoříme, až bude showKeyboard true
	});
  
	$: if (showKeyboard) {
	  // Když se showKeyboard změní na true, inicializujeme klávesnici
	  tick().then(() => {
		if (keyboardContainer && !keyboard) {
		  keyboard = new SimpleKeyboard(keyboardContainer, {
			onKeyPress: handleKeyPress,
			physicalKeyboardHighlight: true,
			theme: "hg-theme-default cratec-theme",
			layout: {
			  default: [
				"ě š č ř ž ý á í é -",
				"q w e r t y u i o p {bksp}",
				"a s d f g h j k l",
				"{shift} z x c v b n m .",
				"{space}",
			  ],
			  shift: [
				"1 2 3 4 5 6 7 8 9 0",
				"Q W E R T Y U I O P {bksp}",
				"A S D F G H J K L",
				"{shift} Z X C V B N M",
				"{space}",
			  ],
			},
			display: {
			  "{bksp}": "←",
			  "{space}": "Space",
			},
			maxLength: 12,
		  });
		  setTimeout(addTouchListenersToKeys, 200);
		}
	  });
	}
  
	onDestroy(() => {
	  // Úklid event listenerů
	  if (keyboardContainer) {
		const buttons = keyboardContainer.querySelectorAll(".hg-button");
		buttons.forEach((btn: HTMLElement) => {
		  btn.removeEventListener("touchstart", () => {});
		  btn.removeEventListener("touchend", () => {});
		});
	  }
	});
  
	function closeKeyboard() {
	  // Zavřeme klávesnici
	  dispatch("closeKeyboard");
	}
  </script>
  
  {#if showKeyboard}
  <!-- Poloprůhledný overlay, abychom simulovali modal -->
  <div
	class="keyboard-overlay"
	on:click|self={closeKeyboard}  
	style="z-index: 999999; position: fixed; inset: 0; background-color: rgba(0,0,0,0.3);"
  >
	<div class="keyboard-wrapper" style="top: {posY}px; left: {posX}px;" on:click={(e) => e.stopPropagation()}>
	  <div class="keyboard-container">
		<div bind:this={keyboardContainer} class="simple-keyboard"></div>
	  </div>
	  <div class="action-buttons">
		<button class="drag-button" on:pointerdown={onDragStart} on:touchstart={onDragStart}>⇕</button>
		<button class="close-button" on:click={closeKeyboard} on:touchend={closeKeyboard}>✖</button>
	  </div>
	</div>
  </div>
  {/if}
  
  <style>
  .keyboard-container {
	position: relative;
	padding: 0.5rem;
	background-color: rgba(80,80,80,1);
	box-shadow: 0 0 10px rgba(0,0,0,0.2);
	border-radius: 20px;
	clip-path: polygon(0 0, calc(100% - 71px) 0, 100% 55px, 100% 100%, 0 100%);
	overflow: hidden;
  }
  
  .action-buttons {
	position: absolute;
	top: -20px;
	right: -30px;
	width: 80px;
	height: 80px;
  }
  
  .drag-button,
  .close-button {
	background: rgba(120,120,120,1);
	color: #fff;
	border: none;
	border-radius: 50%;
	width: 40px;
	height: 40px;
	font-size: 1.2rem;
	touch-action: manipulation;
	display: flex;
	justify-content: center;
	align-items: center;
  }
  .drag-button { position: absolute; top: 0; }
  .close-button { position: absolute; top: 30px; right: 0; }
  
  :global(.simple-keyboard) {
	background-color: rgba(80,80,80,1);
  }
  :global(.simple-keyboard .hg-button) {
	height: 40px;
	margin: 2px;
	background: rgba(120,120,120,1);
	color: #fff;
	font-size: 1rem;
	border: none;
	border-radius: 4px;
	display: flex;
	justify-content: center;
	align-items: center;
	user-select: none;
	touch-action: manipulation;
  }
  :global(.simple-keyboard .hg-button:active) {
	background: rgba(170,170,170,1);
  }
  </style>
  