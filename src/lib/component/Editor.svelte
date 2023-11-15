<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { wast } from '@codemirror/lang-wast';
	import { invoke } from '@tauri-apps/api/tauri';
	import * as command from '../../bindings';

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
			{#each result.func as f}
				<h2>{f.name}</h2>
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
						<li>Step {i + 1}: {JSON.stringify(instruction)}</li>
					{/each}
				</ul>
				<h3>Return</h3>
				<ul>
					{#each f.result as res, i}
						<li>{i}: {res}</li>
					{/each}
				</ul>
			{/each}
		</section>
		<p class=" bg-success-700">{JSON.stringify(result, null, 4)}</p>
	{:else}
		<p class=" text-black">No Output</p>
	{/if}
</section>
