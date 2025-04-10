import { writable } from 'svelte/store';
import type { DeviceBase } from '$lib/stores/deviceStore';

export interface EwfParams {
	case_number: string;
	description: string;
	investigator_name: string;
	evidence_number: string;
	notes:string;
	bytes_to_read:number[];
	offset:number[];
}

export interface DdParams {
	case_number: string;
	description: string;
	investigator_name: string;
	evidence_number: string;
}

export interface CopyRunState {
	inputDisk: DeviceBase | null;
	outputDisks: DeviceBase[];
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
		evidence_number: '',
		notes: '',
		bytes_to_read: [0],
		offset: [0]
	},
	ddParams: {
		case_number: '',
		description: '',
		investigator_name: '',
		evidence_number: ''
	}
});
