import { writable } from 'svelte/store';
import type { DeviceBase } from '$lib/stores/deviceStore';

export interface Process {
	id: string; // Unikátní identifikátor procesu
	start_datetime: string; // Datum a čas spuštění procesu
	end_datetime: string | null; // Datum a čas ukončení procesu, null pokud stále běží
	status: string; // Aktuální stav procesu (např. 'běží', 'ukončen', 'chyba')
	triggered_by_ewf: boolean; // Indikuje, zda byl proces spuštěn pomocí EWF
	triggered_by_dd: boolean; // Indikuje, zda byl proces spuštěn pomocí DD
	speed: number; // Rychlost zpracování (např. MB/s)
	source_disk: DeviceBase; // Zdrojový disk – nemůže být null
	destination_disks: [DeviceBase, ...DeviceBase[]]; // Pole cílových disků – vždy alespoň jeden
	progress_perc: number; // Procentuální dokončení procesu
	progress_time: number; // Doba zpracování (např. v sekundách)
	out_log: string[]; // Pole pro zaznamenání postupu procesu. Logy se ukládají ve stejném pořadí, v jakém jsou vkládány.
}

export const runningProcessesStore = writable<Process[]>([]);
