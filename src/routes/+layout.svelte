<script lang="ts">
	import '../app.postcss';
	import { Toast, initializeStores, LightSwitch, Modal } from '@skeletonlabs/skeleton';
	import type { ModalComponent } from '@skeletonlabs/skeleton';
	import NewConfigEwfModal from '$lib/components/modals/NewConfigEwfModal.svelte';
	import EwfConfigDetailModal from '$lib/components/modals/EwfConfigDetailModal.svelte';
	import DiskSelectModal from '$lib/components/modals/DiskSelectModal.svelte';
	import NewConfigDDModal from '$lib/components/modals/NewConfigDDModal.svelte';

	initializeStores();

	import { computePosition, autoUpdate, offset, shift, flip, arrow } from '@floating-ui/dom';

	import { storePopup } from '@skeletonlabs/skeleton';
	storePopup.set({ computePosition, autoUpdate, offset, shift, flip, arrow });

	const modalRegistry: Record<string, ModalComponent> = {
		NewConfigEwfModal: { ref: NewConfigEwfModal },
		DiskSelectModal: { ref: DiskSelectModal },
		EwfConfigDetailModal: { ref: EwfConfigDetailModal },
		NewConfigDDModal: { ref: NewConfigDDModal }
	};
</script>

<Modal components={modalRegistry} />
<Toast position="t" />

<slot></slot>
