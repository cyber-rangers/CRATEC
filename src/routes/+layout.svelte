<script lang="ts">
	import '../app.postcss';
	import { Toast, initializeStores, LightSwitch, Modal } from '@skeletonlabs/skeleton';
	import type { ModalComponent } from '@skeletonlabs/skeleton';
	import FormModal from '../components/modals/FormModal.svelte';
	initializeStores();

	import { computePosition, autoUpdate, offset, shift, flip, arrow } from '@floating-ui/dom';

	import { storePopup } from '@skeletonlabs/skeleton';
	storePopup.set({ computePosition, autoUpdate, offset, shift, flip, arrow });

	const modalRegistry: Record<string, ModalComponent> = {
		FormModal: { ref: FormModal }
	};
</script>

<Modal components={modalRegistry} />
<Toast position="t" />

<slot></slot>
