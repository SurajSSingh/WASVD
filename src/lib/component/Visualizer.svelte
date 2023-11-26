<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
	import { Presentation, Slide } from '$lib/animotion/components';
	import { deserialize_number } from '$lib';
	import { exec_instructions, type EvalResult, type MyError, type WasmData } from '$lib/interpreter';
    import type * as command from '$lib/bindings';
    import { watStructure } from "$lib/store";
	import Simulator from './Simulator.svelte';
	import { Accordion, AccordionItem } from '@skeletonlabs/skeleton';

    let funcSelect: number = -1;
    let steps: { result: EvalResult; previous: (number | bigint)[]; current: (number | bigint)[] }[] =
		[];
	let stepError: MyError | null = null;
    let data: WasmData = {
        globals: {},
        memory: {},
        functions: [],
    };
    let params: {[key:string]:number} = {};

    function reset(){
        funcSelect = -1;
        steps = [];
        stepError = null;
    }

    let unsubscribe = watStructure.subscribe((structure) => {
        reset();
        for (const global of structure?.globals??[]) {
            data.globals[global.name] = deserialize_number(global.val);
        }
        for (const memory of structure?.memory??[]) {
            data.memory[memory.name] = memory.data;
        }
    })
    onDestroy(() => {
        unsubscribe()
    })

    function run(func: command.WastFunc) {
        // let locals = func.info.input.map(())
        let local_names = func.info.input.concat(func.locals);
		const result = exec_instructions(func.block, data, {});
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
	// let Slide: any;

	// onMount(async () => {
	// 	Animation = (await import("$lib/animotion/components/index"));
	// 	Presentation = Animation.Presentation;
	// 	Slide = Animation.Slide;
	// 	console.log(Presentation);
	// })
</script>

<div class="h-full overflow-auto">
    <h2 class="text-center">Visualizer</h2>
    <Accordion>
        <AccordionItem>
            <!-- <svelte:fragment slot="lead">(icon)</svelte:fragment> -->
			<svelte:fragment slot="summary">Globals</svelte:fragment>
			<svelte:fragment slot="content">
                <section class="card">
                    <!-- <h3 class="card-header">Globals</h3> -->
                    {#if $watStructure}
                        <div class="table-container">
                            <table class="table table-hover">
                                <thead>
                                    <tr>
                                        <th>Index</th>
                                        <th>Name</th>
                                        <th>Type</th>
                                        <th>Is Mutable?</th>
                                        <th>Value</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each $watStructure.globals as g, i}
                                        <tr>
                                            <td>{i}</td>
                                            <td>{g.name}</td>
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
			</svelte:fragment>
        </AccordionItem>
        <AccordionItem>
            <!-- <svelte:fragment slot="lead">(icon)</svelte:fragment> -->
			<svelte:fragment slot="summary">Memory</svelte:fragment>
			<svelte:fragment slot="content">
                <section class="card">
                    {#if $watStructure}
                    <div class="table-container">
                        <table class="table table-hover">
                            <thead>
                                <tr>
                                    <th>Index</th>
                                    <th>Name</th>
                                    <th>Is 32-bit</th>
                                    <th>Is Shared</th>
                                    <th>Min Size</th>
                                    <th>Max Size</th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each Object.entries($watStructure.exported) as [name, [kind, index]], _ (name)}
                                {#if kind === 'Memory'}
                                {@const m = $watStructure.memory.at(index)}
                                {#if m}
                                <tr>
                                    <td>{index}</td>
                                    <td>{m.name}</td>
                                    <td>{m.is_32}</td>
                                    <td>{m.is_shared}</td>
                                    <td>
                                        {deserialize_number(m.min)}
                                    </td>
                                    <td>
                                        {deserialize_number(m.max)}
                                    </td>
                                </tr>
                                {/if}
                                {/if}
                                {/each}
                            </tbody>
                        </table>
                    </div>
                    {/if}
                </section>
			</svelte:fragment>
        </AccordionItem>
        <AccordionItem>
            <!-- <svelte:fragment slot="lead">(icon)</svelte:fragment> -->
			<svelte:fragment slot="summary">Exported Functions</svelte:fragment>
			<svelte:fragment slot="content">
                <section class="card">
                    {#if $watStructure}
                    <div class="table-container">
                        <table class="table table-hover">
                            <thead>
                                <tr>
                                    <th>Name</th>
                                    <th>Inputs</th>
                                    <th>Locals</th>
                                    <th>Instruction Count</th>
                                    <th>Results</th>
                                </tr>
                            </thead>
                        <tbody>
                            {#each Object.entries($watStructure.exported) as [name, [kind, index]], _ (name)}
                            {#if kind === 'Function'}
                            {@const f = $watStructure.func.at(index)}
                            {#if f}
                                <tr>
                                    <td>{f.info.index}</td>
                                    <td>{JSON.stringify(f.info.input)}</td>
                                    <td>{JSON.stringify(f.locals)}</td>
                                    <td>{f.block.array.length}</td>
                                    <td>{JSON.stringify(f.info.output)}</td>
                                </tr>
                            {/if}
                            {/if}
                            {/each}
                        </tbody>
                    </table>
                </div>
                {/if}
                </section>
			</svelte:fragment>
        </AccordionItem>
    </Accordion>
    <h3>Function Simulator</h3>
    <label class="label">
        <span>Choose Function</span>
        <select bind:value={funcSelect} class="select">
            {#if $watStructure}
                {#each Object.entries($watStructure.exported) as [name, [kind, index]], _ (name)}
                    {#if kind === 'Function'}
                        {@const f = $watStructure.func.at(index)}
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
    {#if $watStructure && funcSelect > -1}
        {@const f = $watStructure.func.at(funcSelect)}
        {#if f}
            <div class="grid grid-cols-2 py-4">
                <section class="p-4 overflow-auto">
                    <h3>Parameters</h3>
                    {#each f.info.input as p, index}
                        {@const name = p[0] ? p[0] + `(${index})` : index.toString()}
                        <label class="label">
                            <span>Parameter {name} : {p[1]} = </span>
                            <input id="number" type="number" bind:value="{params[name]}" class=" bg-slate-800" />
                        </label>
                    {/each}
                </section>
                <section class="p-4 overflow-auto">
                    <h3>Results</h3>
                    {#each f.info.output as r, index}
                    <span>Result {index} : {r} = {steps?.slice(-1)[0]?.current[index] ?? "???"}
                    </span>
                {/each}
                </section>
            </div>
            <button on:click={() => run(f)} class="btn btn-md bg-primary-500">Run</button>
            <hr />
            {#if steps.length > 0}
            <div class="h-1/3">
                <Simulator steps={steps}/>
            </div>
            {/if}
            {#if stepError}
                 <p>{stepError.message}</p>
            {/if}
        {/if}
    {/if}
</div>