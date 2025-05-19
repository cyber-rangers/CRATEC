// src/lib/stores/deviceStore.ts
import { persisted } from 'svelte-persisted-store';

export interface DeviceBase {
	interface: string;
	serial: string | null;
	name: string | null;
	sector_count: number | null;
	sector_size: number | null;
	side: string | null;
	type: 'usb' | 'sata';
	mountpoint: string | null;
}

export interface DeviceStatus {
	usb_devices: DeviceBase[];
	sata_devices: DeviceBase[];
	cpu_usage: number;
	ram_usage: number;
}

export const deviceStore = persisted<DeviceStatus>('deviceStore', {
	usb_devices: [],
	sata_devices: [],
	cpu_usage: 0,
	ram_usage: 0
});
