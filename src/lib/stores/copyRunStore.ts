import { writable } from 'svelte/store';

export interface Disk {
	interface: string;
	type: string;
}

export interface CopyRunState {
	inputDisk: Disk | null;
	outputDisks: Disk[];
}

export const copyRunStore = writable<CopyRunState>({
	inputDisk: null,
	outputDisks: []
});
