import { writable } from 'svelte/store';

export interface Disk {
	interface: string;
	type: string;
}

export interface EwfParams {
	case_number: string;
	description: string;
	investigator_name: string;
	evidence_number: string;
}

export interface DdParams {
	case_number: string;
	description: string;
	investigator_name: string;
	evidence_number: string;
}

export interface CopyRunState {
	inputDisk: Disk | null;
	outputDisks: Disk[];
	ewfParams: EwfParams;
	ddParams: DdParams;
}

export const copyRunStore = writable<CopyRunState>({
	inputDisk: null,
	outputDisks: [],
	ewfParams: {
		case_number: '',
		description: '',
		investigator_name: '',
		evidence_number: ''
	},
	ddParams: {
		case_number: '',
		description: '',
		investigator_name: '',
		evidence_number: ''
	}
});
