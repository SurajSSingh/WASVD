<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { wast } from '@codemirror/lang-wast';
	import { invoke } from '@tauri-apps/api/tauri';
	import * as command from '$lib/bindings';
	import { combine_bytes, eval_instructions, instruction_in_plain_english } from '$lib';
	import { TabGroup, Tab } from '@skeletonlabs/skeleton';

	let text: string = '(module)';
	let result: command.InterpreterStructure | null = null;
	let tabSet: number = 0;
	let error: object | null = null;
	let funcSelect: number = -1;
	let globalsList: number[] = [];
	let globalValues = {};

	async function compile() {
		command
			.transform(text)
			.then((res) => {
				error = null;
				result = res;
				for (let index = 0; index < result.globals.length; index++) {
					const global = result.globals[index];
					globalsList[index] = 0;
				}
			})
			.catch((err) => {
				result = null;
				error = err;
			});
	}
</script>

<div class="h-screen w-full">
	<div class="grid grid-cols-2 h-full">
		<div class="h-full overflow-auto">
			<button on:click={compile} class="bg-primary-400">Compile</button>
			<TabGroup>
				<Tab bind:group={tabSet} name="tab1" value={0}>
					<span>Editor</span>
				</Tab>
				<Tab bind:group={tabSet} name="tab2" value={1}>Exported View</Tab>
				<Tab bind:group={tabSet} name="tab3" value={2}>Raw Structure</Tab>
				<!-- Tab Panels --->
				<svelte:fragment slot="panel">
					{#if tabSet === 0}
						<CodeMirror bind:value={text} lang={wast()} class=" bg-slate-100 text-black" />
					{:else if tabSet === 1}
						<section class=" bg-slate-100">
							{#if result}
								<section class=" text-black">
									{#each Object.entries(result.exported) as [name, [kind, index]], _ (name)}
										{#if kind === 'Function'}
											{@const f = result.func.at(index)}
											{#if f}
												<h2>Function {f.name ?? name}</h2>
												<h3>Parameters</h3>
												<ul>
													{#each f.parameters as param, i}
														<li>{param[0] ?? i}: {param[1]}</li>
													{/each}
												</ul>
												<h3>Locals</h3>
												<ul>
													{#each f.locals as loc, i}
														<li>{loc[0] ?? i}: {loc[1]}</li>
													{/each}
												</ul>
												<h3>Instructions</h3>
												<ul>
													{#each f.body as instruction, i}
														<li>
															<strong>Step {i + 1}</strong>: {instruction_in_plain_english(
																instruction
															)}
														</li>
													{/each}
												</ul>
												<h3>Return</h3>
												<ul>
													{#each f.result as res, i}
														<li>{i}: {res}</li>
													{/each}
												</ul>
											{:else}
												<h1 class=" bg-error-500">
													No function at index {index} for exported name {name}
												</h1>
											{/if}
										{:else if kind === 'Global'}
											<h1>Global {name}</h1>
										{:else if kind === 'Memory'}
											{@const m = result.memory.at(index)}
											{#if m}
												<h2>Memory {m.name} (exported as {name})</h2>
												<h3>Properties</h3>
												<ul>
													<li>Is 32-bit: {m.is_32}</li>
													<li>Min Size: {combine_bytes(m.min_lower, m.min_upper)}</li>
													<li>Max Size: {combine_bytes(m.max_lower, m.max_upper)}</li>
													<li>Is Shared: {m.is_shared}</li>
												</ul>
												<h3>Current Data</h3>
												<p>{m.data}</p>
											{:else}
												<h1 class=" bg-error-500">
													No function at index {index} for exported name {name}
												</h1>
											{/if}
										{/if}
									{/each}
								</section>
							{:else}
								<p class="text-black">No Output (Compile code first)</p>
							{/if}
						</section>
					{:else if tabSet === 2}
						<section class=" bg-slate-100">
							{#if error}
								<p class=" bg-error-700">{JSON.stringify(error, null, 4)}</p>
							{:else if result}
								<p class=" bg-success-700">{JSON.stringify(result, null, 4)}</p>
							{:else}
								<p class=" text-black">No Output (Compile code first)</p>
							{/if}
						</section>
					{/if}
				</svelte:fragment>
			</TabGroup>
		</div>
		<div class="h-full overflow-auto">
			<h2>Visualizer</h2>
			<span>Selected: {funcSelect}</span>
			<section class="card">
				<h3 class="card-header">Globals</h3>
				{#if result}
					<span>{JSON.stringify(result.globals)}</span>
					<div class="table-container">
						<table class="table table-hover">
							<thead>
								<tr>
									<th>Name</th>
									<th>Type</th>
									<th>Is Mutable?</th>
									<th>Value</th>
								</tr>
							</thead>
							<tbody>
								{#each result.globals as g}
									<tr>
										<td>{g.name} ()</td>
										<td>{g.typ}</td>
										<td>{g.is_mutable}</td>
										<td>
											{eval_instructions(
												g.val,
												{},
												{ indexToName: [], names: {} },
												{ indexToName: [], names: {} }
											)}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</section>
			<section class="card">
				{#if result}
					{#each Object.entries(result.exported) as [name, [kind, index]], _ (name)}
						{#if kind === 'Memory'}
							{@const f = result.func.at(index)}
							{#if f}
								<option value={index}>{f.name ?? name}</option>
							{/if}
						{/if}
					{/each}
				{/if}
			</section>
			<label class="label">
				<span>Choose Function</span>
				<select bind:value={funcSelect} class="select">
					{#if result}
						{#each Object.entries(result.exported) as [name, [kind, index]], _ (name)}
							{#if kind === 'Function'}
								{@const f = result.func.at(index)}
								{#if f}
									<option value={index}>{name}</option>
								{/if}
							{/if}
						{:else}
							<option value="-1">No items exported</option>
						{/each}
					{:else}
						<option value="-1">Please Compile code to select</option>
					{/if}
				</select>
			</label>
			{#if result && funcSelect > -1}
				{@const f = result.func.at(funcSelect)}
				{#if f}
					{#each f.parameters as p, index}
						{@const name = p[0] ? p[0] + `(${index})` : index.toString()}
						<label class="label">
							<span>Parameter {name} : {p[1]} = </span>
							<input id="number" type="number" value="0" class=" bg-slate-800" />
						</label>
					{/each}
					<hr />

					<hr />
					{#each f.result as r, index}
						<span>Result {index} : {r} = {0}</span>
					{/each}
				{/if}
			{/if}
		</div>
	</div>
</div>
