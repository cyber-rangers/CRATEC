<script lang="ts">
  import SimpleKeyboard from "simple-keyboard";
  import "simple-keyboard/build/css/index.css";
  import { onMount, onDestroy, createEventDispatcher } from "svelte";

  export let activeInput: string;
  export let formData: Record<string, string>;

  const dispatch = createEventDispatcher();
  let keyboard: SimpleKeyboard;

  // Výchozí pozice klávesnice – horizontálně vystředěná, cca 300px od spodní části
  let posX = window.innerWidth / 2 - (window.innerWidth * 0.7) / 2;
  let posY = window.innerHeight - 300;

  // Proměnné pro logiku tažení (drag)
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

  // Zpracování stisku kláves – vkládá znak do příslušného input pole
  function handleKeyPress(button: string) {
    if (!activeInput) return;
    
    if (button === "{bksp}") {
      formData[activeInput] = formData[activeInput]?.slice(0, -1) || "";
    } else if (button === "{space}") {
      formData[activeInput] += " ";
    } else {
      formData[activeInput] += button;
    }
    
    const input = document.querySelector(`[name="${activeInput}"]`) as HTMLInputElement;
    if (input) {
      input.value = formData[activeInput];
      input.focus();
      const end = input.value.length;
      input.setSelectionRange(end, end);
    }
  }

  // Přidání vlastních dotykových posluchačů k tlačítkům SimpleKeyboard
  function addTouchListenersToKeys() {
    const buttons = document.querySelectorAll(".simple-keyboard .hg-button");
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

  // Funkce pro zavření klávesnice – vyvolá událost, kterou zachytí nadřazená komponenta
  function handleClose(e) {
    e.preventDefault();
    dispatch("closeKeyboard");
  }

  onMount(() => {
    keyboard = new SimpleKeyboard({
      onKeyPress: handleKeyPress,
      physicalKeyboardHighlight: true,
      theme: "hg-theme-default cratec-theme",
      layout: {
        default: [
          "1 2 3 4 5 6 7 8 9 0",
          "q w e r t y u i o p {bksp}",
          "a s d f g h j k l",
          "z x c v b n m",
          "{space}"
        ]
      },
      display: {
        "{bksp}": "⌫",
        "{space}": "Space"
      }
    });
    setTimeout(addTouchListenersToKeys, 200);
  });

  onDestroy(() => {
    const buttons = document.querySelectorAll(".simple-keyboard .hg-button");
    buttons.forEach((btn: HTMLElement) => {
      btn.removeEventListener("touchstart", () => {});
      btn.removeEventListener("touchend", () => {});
    });
  });
</script>

<!-- Struktura:
     - Obal (.keyboard-wrapper) je pozicován pomocí inline stylů (top, left).
     - V něm je element s oválným pozadím (s clip-path), který představuje klávesnici.
     - Akční tlačítka jsou umístěna uvnitř kontejneru .action-buttons s absolutním pozicováním,
       takže jedno tlačítko (drag) je umístěno v pravém horním rohu a druhé (close) je posunuto
       diagonálně dolů a doleva. -->
<div class="keyboard-wrapper" style="top: {posY}px; left: {posX}px;">
  <div class="keyboard-container">
    <div class="simple-keyboard"></div>
  </div>
  <div class="action-buttons">
    <button class="drag-button"
      on:pointerdown={onDragStart}
      on:touchstart={onDragStart}>
      ⇕
    </button>
    <button class="close-button" on:click={handleClose} on:touchend={handleClose}>
      ✖
    </button>
  </div>
</div>

<style>
  /* Obal klávesnice – pozicovaný fixed podle posX/posY */
  .keyboard-wrapper {
    position: fixed;
    z-index: 1000;
    width: 70vw;
  }
  /* Kontejner s pozadím klávesnice – oválný tvar s "výkrojem" v pravém horním rohu */
  .keyboard-container {
    position: relative;
    padding: 0.5rem;
    background-color: rgba(var(--color-surface-700) / 1);
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
    border-radius: 20px;
    clip-path: polygon(
      0 0,
      calc(100% - 71px) 0,
      100% 55px,
      100% 100%,
      0 100%
    );
    overflow: hidden;
  }
  /* Kontejner pro akční tlačítka – absolutně pozicován uvnitř obalu klávesnice */
  .action-buttons {
    position: absolute;
    top: -20px;
    right: -30px;
    width: 80px;  /* nastavit dle potřeby */
    height: 80px; /* nastavit dle potřeby */
  }
  /* Styl pro obě tlačítka */
  .drag-button,
  .close-button {
    background: rgba(var(--color-surface-500) / 1);
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
  /* Drag tlačítko – umístěno v pravém horním rohu */
  .drag-button {
    position: absolute;
    top: 0;
    
  }
  /* Close tlačítko – umístěno diagonálně níže (posunuto dolů a doleva) */
  .close-button {
    position: absolute;
    top: 30px;
    right: 0px;
  }
  :global(.simple-keyboard) {
    background-color: rgba(var(--color-surface-700) / 1);
  }
  :global(.simple-keyboard .hg-button) {
    height: 40px;
    margin: 2px;
    background: rgba(var(--color-surface-500) / 1);
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
    background: rgba(var(--color-surface-300) / 1);
  }
</style>
