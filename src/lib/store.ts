import { writable, type Writable } from "svelte/store";
import type * as command from '$lib/bindings';
import type Reveal from "reveal.js";

const watStructure: Writable<command.InterpreterStructure | null> = writable(null);
const compErr: Writable<command.WatError | null> = writable(null);
const revealDeck: Writable<Reveal.Api | null> = writable(null);
export {watStructure, compErr, revealDeck}