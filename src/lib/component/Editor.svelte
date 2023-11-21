<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { wast } from '@codemirror/lang-wast';
	import * as command from '$lib/bindings';
	import {
		deserialize_number,
	} from '$lib';
	import { TabGroup, Tab } from '@skeletonlabs/skeleton';
	import InstructionBlock from './InstructionBlock.svelte';
	import {watStructure} from "$lib/store"

	let text: string = `(module
	(func $example (export "example") (result i32)
		i32.const 1
		i32.const 2
		i32.add
	)
)`;
	let tabSet: number = 0;
	let compError: object | null = null;

	function reset(){
		watStructure.set(null);
		compError = null;
	}

	async function compile() {
		reset();
		command
			.transform(text)
			.then((res) => {
				console.log(res);
				watStructure.set(res);
			})
			.catch((err) => {
				compError = err;
			});
	}

</script>


<div class="h-full overflow-auto">
	<button on:click={compile} class="bg-primary-400 p-2">Compile</button>
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
				<section class=" bg-slate-100 m-1 p-1">
					{#if $watStructure}
						<section class=" text-black">
							{#each Object.entries($watStructure.exported) as [name, [kind, index]], _ (name)}
								{#if kind === 'Function'}
									{@const f = $watStructure.func.at(index)}
									{#if f}
										<h2>Function {f.info.index ?? name}</h2>
										<h3>Parameters</h3>
										<ul>
											{#each f.info.input as param, i}
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
										<InstructionBlock nodes={f.block.root} />
										<h3>Return</h3>
										<ul>
											{#each f.info.output as res, i}
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
									{@const m = $watStructure.memory.at(index)}
									{#if m}
										<h2>Memory {m.name} (exported as {name})</h2>
										<h3>Properties</h3>
										<ul>
											<li>Is 32-bit: {m.is_32}</li>
											<li>Min Size: {deserialize_number(m.min)}</li>
											<li>Max Size: {deserialize_number(m.max)}</li>
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
					{#if compError}
						<p class=" bg-error-700">{JSON.stringify(compError, null, 4)}</p>
					{:else if $watStructure}
						<p class=" bg-success-700">{JSON.stringify($watStructure, null, 4)}</p>
					{:else}
						<p class=" text-black">No Output (Compile code first)</p>
					{/if}
				</section>
			{/if}
		</svelte:fragment>
	</TabGroup>
</div>
