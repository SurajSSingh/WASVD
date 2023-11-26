<script lang="ts">
	import { deserialize_number } from "$lib";
    import {watStructure, compErr} from "$lib/store"
	import InstructionBlock from "$lib/component/InstructionBlock.svelte";

</script>

<div class=" bg-slate-100 m-1 p-1">
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
                        <InstructionBlock tree={f.block} index={0}/>
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
    {:else if $compErr}
    <section class=" text-black">
        <p class="text-error-500">Error occured!</p>
        <p>Kind: {$compErr.stage} @{$compErr.span ? `${$compErr.span.start} to ${$compErr.span.end}` : "Unknown"}</p>
        <p>Message: {$compErr.message}</p>
    </section>
    {:else}
        <p class="text-black">No Output (Compile code first)</p>
    {/if}
</div>