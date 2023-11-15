<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { wast } from '@codemirror/lang-wast';
	import { invoke } from '@tauri-apps/api/tauri';
	import * as command from '$lib/bindings';
	import { combine_bytes, instruction_in_plain_english } from '$lib';

	let text = '(module)';
	let result: command.InterpreterStructure | null = null;
	let error: object | null = null;

	async function compile() {
		command
			.transform(text)
			.then((res) => {
				error = null;
				result = res;
			})
			.catch((err) => {
				result = null;
				error = err;
			});
		// invoke('transform', { text })
		// 	.then((res) => {
		// 		error = null;
		// 		if (typeof res === 'string' || typeof res === 'object') {
		// 			result = res;
		// 		} else {
		// 			console.log(res);
		// 		}
		// 	})
		// 	.catch((err) => {
		// 		result = null;
		// 		error = err;
		// 	});
	}
</script>

<CodeMirror bind:value={text} lang={wast()} class=" bg-slate-100 text-black" />
<button on:click={compile} class="bg-primary-400">Compile</button>
<section class=" bg-slate-100">
	{#if error}
		<p class=" bg-error-700">{JSON.stringify(error, null, 4)}</p>
	{:else if result}
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
								<li><strong>Step {i + 1}</strong>: {instruction_in_plain_english(instruction)}</li>
							{/each}
						</ul>
						<h3>Return</h3>
						<ul>
							{#each f.result as res, i}
								<li>{i}: {res}</li>
							{/each}
						</ul>
					{:else}
						<h1 class=" bg-error-500">No function at index {index} for exported name {name}</h1>
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
						<h1 class=" bg-error-500">No function at index {index} for exported name {name}</h1>
					{/if}
				{/if}
			{/each}
		</section>
		<p class=" bg-success-700">{JSON.stringify(result, null, 4)}</p>
	{:else}
		<p class=" text-black">No Output</p>
	{/if}
</section>
