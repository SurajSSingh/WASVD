<script lang="ts">
	import type { SVGStackObjData, SignalResult, StackSVGInitializer } from "$lib/stackAnim";
	import SignalStackBox from "./SignalStackBox.svelte";
    import StackBox from "./StackBox.svelte";
    export let working1: SignalResult<SVGStackObjData>;
    export let working2: SignalResult<SVGStackObjData>;
    export let operator: SignalResult<SVGStackObjData>;
    export let stacks: SignalResult<SVGStackObjData>[];
    export let restOf: SignalResult<{opacity: 0 | 1}>;
    export let restData: Partial<SVGStackObjData>;
    const restValue = "...";
    const restdefaultData: SVGStackObjData = {
        opacity: 1,
        xPosition: 0,
        yPosition: 0,
        width: 0,
        height: 0,
        fill: "black",
        text: "white",
        stroke: "red",
        value:restValue
    };
                    
    $: finalSignal = stacks[0];
</script>

<svg class="w-full h-[400px] mx-auto bg-primary-800" viewBox="0 0 800 800">
    <!-- Working Boxes -->
    <StackBox data={$working1}/>
    <StackBox data={$working2}/>
    <StackBox data={$operator}/>
    <!-- Stack Boxes -->
    {#each stacks as stackSignal}
        <SignalStackBox signal={stackSignal}/>
    {/each}
    <!-- Rest of Stack -->
    <!-- TODO: Make rest of stack only dependent on "restOf" value -->
    <StackBox data={{...restdefaultData, ...restData, value:restValue, ...$restOf}}/>
</svg>