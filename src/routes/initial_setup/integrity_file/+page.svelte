<script lang="ts">
	import { onMount } from 'svelte';
	import { Stepper, Step, ProgressRadial } from '@skeletonlabs/skeleton';
	import { invoke } from '@tauri-apps/api/tauri';
	import { goto } from '$app/navigation';

	// Interface Definitions
	interface Hashes {
		MD5: string;
		SHA1: string;
		SHA2: string;
	}

	interface Versions {
		[version: string]: Hashes;
	}

	interface IntegrityData {
		ver: string;
		[softwareName: string]: Versions | string;
	}

	interface TableRow {
		softwareName: string;
		version: string;
		MD5: string;
		SHA1: string;
		SHA2: string;
	}

	// Variable Declarations
	let findFileProgress: number | undefined = 0;
	let fetchDataProgress: number | undefined = 0;
	let verifyCompatibilityProgress: number | undefined = 0;
	let checkIntegrityProgress: number | undefined = 0;

	let stepLock: boolean = true;

	let integrityData: IntegrityData = { ver: '' };
	let tableData: TableRow[] = [];
	let integrityValid = false;

	onMount(() => {
		startProcess();
	});

	function onCompleteHandler(e: Event): void {
		goto('/dashboard');
	}

	async function startProcess() {
		try {
			findFileProgress = undefined;
			const fileExists = await invoke('find_file');
			findFileProgress = fileExists ? 100 : 0;
			if (!fileExists) {
				throw new Error('Soubor integrity.crkcfg nebyl nalezen.');
			}

			checkIntegrityProgress = undefined;
			const result = await invoke('check_integrity');
			checkIntegrityProgress = 100;

			if (typeof result === 'boolean' && !result) {
				integrityValid = false;
				console.error('Ověření integrity selhalo. Data mohla být změněna.');
			} else {
				integrityValid = true;
				console.log('Integrity check passed. Data:', JSON.stringify(result, null, 2));
			}

			fetchDataProgress = undefined;
			integrityData = (await invoke('fetch_integrity_data')) as IntegrityData;
			fetchDataProgress = 100;

			for (let softwareName in integrityData) {
				if (softwareName !== 'ver') {
					const versions = integrityData[softwareName] as Versions;
					for (let version in versions) {
						const hashes = versions[version];
						tableData.push({
							softwareName,
							version,
							MD5: hashes.MD5,
							SHA1: hashes.SHA1,
							SHA2: hashes.SHA2
						});
					}
				}
			}

			verifyCompatibilityProgress = undefined;
			await invoke('verify_compatibility');
			verifyCompatibilityProgress = 100;

			stepLock = false;
		} catch (error) {
			console.error('An error occurred during the process:', error);
			stepLock = false;
		}
	}
</script>

<div class="h-screen flex flex-col items-center justify-center">
	<Stepper on:complete={onCompleteHandler}>
		<Step locked={stepLock}>
			<svelte:fragment slot="header">
				<span
					class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone"
				>
					File Initialization
				</span>
			</svelte:fragment>
			<div class="step-window">
				<div class="flex flex-col justify-start font-bold">
					<div class="flex items-center mb-4">
						<ProgressRadial
							value={findFileProgress}
							meter="stroke-primary-500"
							width="w-12"
							stroke={70}
						/>
						<span class="progress-text text-xl ml-4">
							Finding File
							<span
								class="bg-gradient-to-br from-primary-800 to-primary-500 bg-clip-text text-transparent box-decoration-clone"
							>
								integrity.crkcfg
							</span>
						</span>
					</div>
					<div class="line"></div>

					<div class="flex items-center mb-4 mt-4">
						<ProgressRadial
							value={checkIntegrityProgress}
							meter="stroke-primary-500"
							width="w-12"
							stroke={70}
						/>
						<span class="text-xl ml-4">Checking Integrity</span>
					</div>
					<div class="line"></div>

					<div class="flex items-center mb-4 mt-4">
						<ProgressRadial
							value={fetchDataProgress}
							meter="stroke-primary-500"
							width="w-12"
							stroke={70}
						/>
						<span class="text-xl ml-4">Fetching data</span>
					</div>
					<div class="line"></div>

					<div class="flex items-center mb-4 mt-4">
						<ProgressRadial
							value={verifyCompatibilityProgress}
							meter="stroke-primary-500"
							width="w-12"
							stroke={70}
						/>
						<span class="text-xl ml-4">Verifying Compatibility</span>
					</div>
				</div>
			</div>
		</Step>

		<Step>
			<svelte:fragment slot="header">
				
			</svelte:fragment>
			<div class="step-window flex justify-around items-center">
				<div style="width: 50rem; height: 32rem;">
					<div class="table-container">
						<table class="table table-compact">
							<thead>
								<tr>
									<th>Software Name</th>
									<th>Version</th>
									<th>Hash type</th>
									<th>Hash</th>
								</tr>
							</thead>
							<tbody>
								{#each tableData as row}
									<tr>
										<td>{row.softwareName}</td>
										<td>{row.version}</td>
										<td>MD5</td>
										<td>{row.MD5}</td>
									</tr>
									<tr>
										<td>{row.softwareName}</td>
										<td>{row.version}</td>
										<td>SHA1</td>
										<td>{row.SHA1}</td>
									</tr>
									<tr>
										<td>{row.softwareName}</td>
										<td>{row.version}</td>
										<td>SHA2</td>
										<td>{row.SHA2}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			</div>
		</Step>
	</Stepper>
</div>

<style lang="postcss">
	.h-screen {
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.step-window {
		width: 50rem;
		height: 25rem;
		display: flex;
		justify-content: space-around;
		align-items: center;
	}

	.ml-4 {
		margin-left: 1rem;
	}

	.line {
		width: 5px;
		height: 20px;
		background-color: white;
		margin: 4px 0;
		align-self: flex-start;
		margin-left: 22px;
		border-radius: 5px;
	}

	.mb-4 {
		margin-bottom: 1rem;
	}

	.mt-4 {
		margin-top: 1rem;
	}

	.card {
		background-color: #1a202c;
		border-radius: 0.5rem;
		box-shadow: 0 10px 15px rgba(0, 0, 0, 0.1);
	}

	@media (max-width: 768px) {
		.step-window {
			width: 90%;
			flex-direction: column;
			align-items: center;
		}

		.card {
			width: 100%;
		}
	}
</style>
