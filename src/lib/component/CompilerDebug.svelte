<script lang="ts">
    import {watStructure} from "$lib/store"
    export let compError:  object | null = null;

</script>

<section class=" bg-slate-100">
    {#if compError}
        <p class=" bg-error-700">{JSON.stringify(compError, null, 4)}</p>
    {:else if $watStructure}
        <p class=" bg-success-700">{JSON.stringify($watStructure, null, 4)}</p>
    {:else}
        <p class=" text-black">No Output (Compile code first)</p>
    {/if}
</section>