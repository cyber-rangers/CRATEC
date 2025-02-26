// src/lib/stores/deviceStore.ts
import { persisted } from 'svelte-persisted-store';

interface SataDevice {
    interface: string;
}

interface UsbDevice {
    interface: string;
}

export interface DeviceStatus {
    usb_devices: UsbDevice[];
    sata_devices: SataDevice[];
    cpu_usage: number;
    ram_usage: number;
}

export const deviceStore = persisted<DeviceStatus>('deviceStore', {
    usb_devices: [],
    sata_devices: [],
    cpu_usage: 0,
    ram_usage: 0
});
