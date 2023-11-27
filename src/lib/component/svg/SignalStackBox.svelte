<script lang="ts">
	import type { SVGStackObjData, SignalResult } from "$lib/stackAnim";
	import StackBox from "./StackBox.svelte";

    export let signal: SignalResult<SVGStackObjData>;
</script>

<StackBox data={$signal}/>