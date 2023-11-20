<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { wast } from '@codemirror/lang-wast';
	import { invoke } from '@tauri-apps/api/tauri';
	import * as command from '$lib/bindings';
	import {
		deserialize_number,
		exec_instructions,
		instruction_in_plain_english,
		type EvalResult,
		type MyError
	} from '$lib';
	import { TabGroup, Tab } from '@skeletonlabs/skeleton';
	import InstructionBlock from './InstructionBlock.svelte';
	import { onMount } from 'svelte';
	// import Pres from './Pres.svelte';
	import { Presentation, Slide } from '$lib/animotion/components';

	let text: string = `(module
	(func $example (export "example") (result i32)
		i32.const 1
		i32.const 2
		i32.add
	)
)`;
	let result: command.InterpreterStructure | null = null;
	let tabSet: number = 0;
	let error: object | null = null;
	let funcSelect: number = -1;
	let globalsList: number[] = [];
	let globalValues = {};
	let steps: { result: EvalResult; previous: (number | bigint)[]; current: (number | bigint)[] }[] =
		[];
	let stepError: MyError | null = null;

	function reset(){
		steps = [];
		stepError = null;
		funcSelect = -1;
		result = null;
		error = null;
	}

	async function compile() {
		reset();
		command
			.transform(text)
			.then((res) => {
				console.log(res);
				result = res;
				for (let index = 0; index < result.globals.length; index++) {
					const global = result.globals[index];
					globalsList[index] = 0;
				}
			})
			.catch((err) => {
				error = err;
			});
	}

	function run(tree: command.SerializedInstructionTree) {
		const result = exec_instructions(tree);
		if ('message' in result) {
			stepError = result;
			steps = [];
		} else {
			stepError = null;
			steps = result;
		}
		console.log(steps);
	}

	// let Animation;
	// let Presentation: any;
	// let Slide;

	onMount(async () => {
		// Animation = (await import("$lib/animotion/components/index"));
		// Presentation = Animation.Presentation;
		// Slide = Animation.Slide;
		// console.log(Presentation);
	})

	function formatStack(stack:(bigint|number)[]):string{
		if(stack.length === 0){
			return "Empty"
		}
		else{
			return stack.map(n => n.toString()).join(", ");
		}
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
						<section class=" bg-slate-100 m-1 p-1">
							{#if result}
								<section class=" text-black">
									{#each Object.entries(result.exported) as [name, [kind, index]], _ (name)}
										{#if kind === 'Function'}
											{@const f = result.func.at(index)}
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
											{@const m = result.memory.at(index)}
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
											{deserialize_number(g.val)}
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
								<option value={index}>{f.info.index ?? name}</option>
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
					{#each f.info.input as p, index}
						{@const name = p[0] ? p[0] + `(${index})` : index.toString()}
						<label class="label">
							<span>Parameter {name} : {p[1]} = </span>
							<input id="number" type="number" value="0" class=" bg-slate-800" />
						</label>
					{/each}
					<button on:click={() => run(f.block)} class="bg-primary-500">Run</button>
					<hr />
					{#if steps.length > 0}
					<p>{steps}</p>
					<Presentation>
						{#each steps as step, i}
						<Slide>
							<p class="font-bold">Step #{i + 1}:</p>
							<p>{step.result.action}</p>
							<p>Start: {formatStack(step.previous)}</p>
							<p>End: {formatStack(step.current)}</p>
						</Slide>
						{/each}
					</Presentation>
					{/if}
					{#if stepError}
						 <p>{stepError.message}</p>
					{/if}
					<hr />
					<section class=" pb-10">
						{#each f.info.output as r, index}
							<span>Result {index} : {r} = {steps?.slice(-1)[0]?.current[index] ?? "???"}
							</span>
						{/each}
					</section>
				{/if}
			{/if}
		</div>
	</div>
</div>
