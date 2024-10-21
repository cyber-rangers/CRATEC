<script lang="ts">
	import { Accordion, AccordionItem } from '@skeletonlabs/skeleton';
	import { FaUsb, FaHdd } from 'svelte-icons/fa';
	import { deviceStore } from '$lib/stores/deviceStore';
	import { get, writable } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/tauri';

	interface SataDevice {
		interface: string;
	}

	interface UsbDevice {
		interface: string;
	}

	let hddDeviceDetails = writable<Record<string, any>>({});
	let usbDeviceDetails = writable<Record<string, any>>({});

	async function fetchDeviceDetails(devicePath: string, isUsb: boolean) {
		console.log(`Fetching details for ${devicePath}, isUsb: ${isUsb}`);
		try {
			const command = isUsb ? 'get_usb_device_details' : 'get_hdd_details';
			const detailedInfo = await invoke(command, { devicePath });
			console.log('Detailed info fetched:', detailedInfo);
			return detailedInfo;
		} catch (error) {
			console.error('Failed to fetch device details:', error);
			return null;
		}
	}

	let fetchTimeout: ReturnType<typeof setTimeout>;

	$: {
		clearTimeout(fetchTimeout);
		fetchTimeout = setTimeout(() => {
			console.log('Device Store Updated:', $deviceStore);
			let sataDevices: SataDevice[] = $deviceStore.sata_devices || [];
			let usbDevices: UsbDevice[] = $deviceStore.usb_devices || [];

			hddDeviceDetails.update((current) => {
				const updated = { ...current };
				for (let path in updated) {
					if (!sataDevices.find((device) => device.interface === path)) {
						delete updated[path];
					}
				}
				return updated;
			});

			usbDeviceDetails.update((current) => {
				const updated = { ...current };
				for (let path in updated) {
					if (!usbDevices.find((device) => device.interface === path)) {
						delete updated[path];
					}
				}
				return updated;
			});

			for (let device of sataDevices) {
				if (!get(hddDeviceDetails)[device.interface]) {
					fetchDeviceDetails(device.interface, false).then((info: any) => {
						if (info) {
							hddDeviceDetails.update((d) => ({ ...d, [device.interface]: info }));
						}
					});
				}
			}

			for (let device of usbDevices) {
				if (!get(usbDeviceDetails)[device.interface]) {
					fetchDeviceDetails(device.interface, true).then((info: any) => {
						if (info) {
							usbDeviceDetails.update((d) => ({ ...d, [device.interface]: info }));
						}
					});
				}
			}
		}, 1000);
	}
</script>

<div class="bg-surface-800 accordion-background">
	<Accordion hover="hover:bg-primary-500">
		{#each $deviceStore.sata_devices || [] as device}
			<AccordionItem>
				<div slot="lead" class="icons">
					<FaHdd />
				</div>
				<div slot="summary">
					HDD {device.interface}
				</div>
				<div slot="content">
					{#if $hddDeviceDetails[device.interface]}
						<h2>{$hddDeviceDetails[device.interface].device_model}</h2>
						<p><strong>Model Family:</strong> {$hddDeviceDetails[device.interface].model_family}</p>
						<p>
							<strong>Serial Number:</strong>
							{$hddDeviceDetails[device.interface].serial_number}
						</p>
						<p>
							<strong>Firmware Version:</strong>
							{$hddDeviceDetails[device.interface].firmware_version}
						</p>
						<p>
							<strong>User Capacity:</strong>
							{$hddDeviceDetails[device.interface].user_capacity}
						</p>
						<p><strong>Sector Sizes:</strong> {$hddDeviceDetails[device.interface].sector_sizes}</p>
						<p>
							<strong>Rotation Rate:</strong>
							{$hddDeviceDetails[device.interface].rotation_rate}
						</p>
						<p><strong>Form Factor:</strong> {$hddDeviceDetails[device.interface].form_factor}</p>
						<p><strong>SMART Status:</strong> {$hddDeviceDetails[device.interface].smart_status}</p>

						<h3>SMART Attributes:</h3>

						<div class="table-container" style="border-bottom-width:2px;">
							<table class="table table-hover" style="width: 100%;">
								<thead class="text-surface-900">
									<tr>
										<th>Attribute Name (ID#)</th>
										<th>Value</th>
										<th>Worst</th>
										<th>Threshold</th>
										<th>Type</th>
										<th>When Failed</th>
										<th>Raw Value</th>
									</tr>
								</thead>
								<tbody>
									{#each $hddDeviceDetails[device.interface].smart_attributes || [] as attr}
										<tr>
											<td>{attr.name} ({attr.id})</td>
											<td>{attr.value}</td>
											<td>{attr.worst}</td>
											<td>{attr.thresh}</td>
											<td>{attr.type_field}</td>
											<td>{attr.when_failed}</td>
											<td>{attr.raw_value}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{:else}
						<p>Loading detailed HDD information...</p>
					{/if}
				</div>
			</AccordionItem>
		{/each}

		{#each $deviceStore.usb_devices || [] as device, index}
			<AccordionItem>
				<div slot="lead" class="icons">
					<FaUsb />
				</div>
				<div slot="summary">
					USB Device {index + 1}: {device.interface}
				</div>
				<div slot="content">
					{#if $usbDeviceDetails[device.interface]}
						<h2>{$usbDeviceDetails[device.interface].id_model}</h2>
						<p><strong>Device Name:</strong> {$usbDeviceDetails[device.interface].devname}</p>
						<p><strong>Device Path:</strong> {$usbDeviceDetails[device.interface].devpath}</p>
						<p><strong>Device Type:</strong> {$usbDeviceDetails[device.interface].devtype}</p>
						<p><strong>Vendor:</strong> {$usbDeviceDetails[device.interface].id_vendor}</p>
						<p><strong>Product:</strong> {$usbDeviceDetails[device.interface].id_model}</p>
						<p>
							<strong>Serial Number:</strong>
							{$usbDeviceDetails[device.interface].id_serial_short}
						</p>
						<p><strong>USB Driver:</strong> {$usbDeviceDetails[device.interface].id_usb_driver}</p>
						<p><strong>Vendor ID:</strong> {$usbDeviceDetails[device.interface].id_vendor_id}</p>
						<p><strong>Subsystem:</strong> {$usbDeviceDetails[device.interface].subsystem}</p>

						<h3>Additional Details:</h3>
						<div class="additional-details">
							<p><strong>Major:</strong> {$usbDeviceDetails[device.interface].major}</p>
							<p><strong>Minor:</strong> {$usbDeviceDetails[device.interface].minor}</p>
							<p><strong>Tags:</strong> {$usbDeviceDetails[device.interface].tags}</p>
							<p>
								<strong>Usec Initialized:</strong>
								{$usbDeviceDetails[device.interface].usec_initialized}
							</p>
						</div>
					{:else}
						<p>Loading detailed USB device information...</p>
					{/if}
				</div>
			</AccordionItem>
		{/each}
	</Accordion>
</div>

<style lang="postcss">
	.accordion-background {
		padding: 20px;
		border-radius: 20px;
	}
	.icons {
		width: 1rem;
		height: 1rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	h2 {
		font-size: 1.5rem;
		margin-bottom: 0.5rem;
	}
	h3 {
		font-size: 1.25rem;
		margin-top: 1rem;
		font-weight: bold;
		margin-bottom: 0.5rem;
	}
	p {
		margin: 0.25rem 0;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		margin-top: 1rem;
	}
	th,
	td {
		border: 1px solid #ddd;
		padding: 0.5rem;
		text-align: left;
	}
	th {
		background-color: #f2f2f2;
	}
	.additional-details {
		margin-top: 1rem;
	}
</style>
