<script lang="ts">
	import Slide from "$lib/animotion/components/slide.svelte";
	import type { SVGStackObjData, SignalResult } from "$lib/stackAnim";
	import StackBox from "./StackBox.svelte";
	import StackSvg from "./StackSVG.svelte";
    export let title: string = "Step";
    export let working1: SignalResult<SVGStackObjData>;
    export let working2: SignalResult<SVGStackObjData>;
    export let operator: SignalResult<SVGStackObjData>;
    export let stacks: SignalResult<SVGStackObjData>[];
    export let restOf: SignalResult<{opacity: 0 | 1}>;
</script>
<Slide>
    <p>{title}</p>
    <StackSvg {working1} {working2} {operator} {stacks} {restOf}/>
</Slide>