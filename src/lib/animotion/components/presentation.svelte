<script lang="ts">
	import { onMount } from 'svelte'
	// import Reveal from 'reveal.js'
	// import options from '$lib/config'

	// import 'reveal.js/dist/reveal.css'
	// import '$lib/animotion/styles/theme.css'
	// import '$lib/animotion/styles/code.css'

	let Reveal;
	let window;
	let options;

	onMount(async () => {
		window = (await import('@tauri-apps/api/window')).default;
		options = (await import("$lib/config")).default;
		Reveal = (await import('reveal.js')).default;

		(await import('reveal.js/dist/reveal.css'));
		(await import('$lib/animotion/styles/theme.css'));
		(await import('$lib/animotion/styles/code.css'));

		// create deck instance
		const deck = new Reveal(options)

		// custom event listeners
		const inEvent = new CustomEvent('in')
		const outEvent = new CustomEvent('out')

		// keep track of current slide
		deck.on('slidechanged', (event) => {
			if ('currentSlide' in event) {
				const currentSlideEl = event.currentSlide as HTMLElement
				currentSlideEl.dispatchEvent(inEvent)
			}

			if ('previousSlide' in event) {
				const currentPreviousEl = event.previousSlide as HTMLElement
				currentPreviousEl.dispatchEvent(outEvent)
			}
		})

		deck.on('fragmentshown', (event) => {
			if ('fragment' in event) {
				const fragmentEl = event.fragment as HTMLElement
				fragmentEl.dispatchEvent(inEvent)
			}
		})

		deck.on('fragmenthidden', (event) => {
			if ('fragment' in event) {
				const fragmentEl = event.fragment as HTMLElement
				fragmentEl.dispatchEvent(outEvent)
			}
		})

		deck.initialize().then(() => {
			// we pass the language to the `<Code>` block
			// and higlight code blocks after initialization
			highlightCodeBlocks(deck)
		})

		// reload page after update to avoid HMR issues
		// reloadPageAfterUpdate()

		console.log("FINISED PRESENTATION MOUNT!");

	})

	function highlightCodeBlocks(deck: Reveal.Api) {
		const highlight = deck.getPlugin('highlight')
		const codeBlocks = [...document.querySelectorAll('code')]
		codeBlocks.forEach((block) => {
			// @ts-ignore
			highlight.highlightBlock(block)
		})
	}

	function reloadPageAfterUpdate() {
		if (import.meta.hot) {
			import.meta.hot.on('vite:afterUpdate', () => {
				location.reload()
			})
		}
	}
</script>

<div class="reveal">
	<div class="slides">
		<slot />
	</div>
</div>
