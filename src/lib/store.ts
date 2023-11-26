import { writable, type Writable } from "svelte/store";
import type * as command from '$lib/bindings';

const watStructure: Writable<command.InterpreterStructure | null> = writable(null);
const compErr: Writable<command.WatError | null> = writable(null);
export {watStructure, compErr}