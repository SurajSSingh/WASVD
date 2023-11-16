// place files you want to import through the `$lib` alias in this folder.
import type * as command from "$lib/bindings"

export function combine_bytes(lower: number, upper:number): bigint | number{
    if(upper === 0){
        return lower
    }
    return (BigInt(upper) << BigInt(32)) | BigInt(lower);
}

export function instruction_in_plain_english(instruction: command.SerializedInstruction):string {
    if (typeof instruction === "string"){
        switch (instruction) {
            case "Unreachable":
                return "Unreachable"
            case "Nop":
                return "Do nothing"
            case "Drop":
                return "Drop the current top value of the stack"
            case "Return":
                return "Return immediately"
            default:
                return JSON.stringify(instruction);
        }
    } else {
        if("Get" in instruction){
            return `Get ${instruction.Get.is_local? "local" : "global"} value '${instruction.Get.loc}' and push it on to the stack.`
        }
        else if("Set" in instruction){
            return `Set ${instruction.Set.is_local? "local" : "global"} value '${instruction.Set.loc}' to the current value on the stack`
        }
        else if("Tee" in instruction){

            return `Set the local value '${instruction.Tee.loc}' to the current value on the stack and immediately push it back onto the stack.`
        }
        else if ("ControlFlow" in instruction){
            const cf = instruction.ControlFlow;
            if("End" in cf){
                return `End of block ${cf.End.length > 0 ? cf.End : ""}`
            }
            else if("Else" in cf){
                return `Else ${cf.Else.length > 0 ? `(label: ${cf.Else})` : ""}`
            }
            else if("Block" in cf){
                const block = cf.Block;
                switch (block.kind) {
                    case "If":
                        return `Start new If${" "+block.label}: If current value on stack is zero, run code, else skip to else.`
                
                    case "Loop":
                        return `Start new loop ${block.label}`
                
                    case "Regular":
                        return `Start new block ${block.label}`
                
                    default:
                        return "UNKNOWN BLOCK"
                }
            }
            else if("Branch" in cf){
                if (cf.Branch.other_labels.length === 0){
                    return "br_table"
                }else if (cf.Branch.is_conditional){
                    return `Jump to ${cf.Branch.default_label} if the value on the stack is not zero.`
                }
                else {
                    return `Jump to ${cf.Branch.default_label}`
                }
            }
            else if("Call" in cf){
                return `Call function ${cf.Call.index}`
            }
        } else if ("Load" in instruction){
            const load = instruction.Load;
            return `Load ${load.typ}`
        } else if ("Store" in instruction){
            const store = instruction.Store;
            return `Store ${store.count}`
        } else if ("Memory" in instruction){
            const memory = instruction.Memory;
            if(memory.will_grow){
                return `Grow memory ${memory.loc} by the current item on the stack and push the old size (success) or -1 (failure) onto the stack.`
            }
            return `Push the size of memory ${memory.loc} onto the stack.`
        } else if ("Const" in instruction){
            const c = instruction.Const;
            return `Push value ${combine_bytes(c.lower32bits, c.upper32bits)} of type ${c.typ} to the stack.`
        } else if ("CountBits" in instruction){
            return `Count ${instruction.CountBits.bit_to_count} for ${instruction.CountBits.is_64 ? 'i64': 'i32'}`
        } else if ("NumericOperation" in instruction){
            const op = instruction.NumericOperation.op;
            if("Comparison" in op){
                return `${op.Comparison} for ${instruction.NumericOperation.typ}`
            } else if ("Arithmetic" in op){
                return `${op.Arithmetic} for ${instruction.NumericOperation.typ}`
            } else if ("Bitwise" in op){
                return `${op.Bitwise} for ${instruction.NumericOperation.typ}`
            } else if ("Float" in op){
                return `${op.Float} for ${instruction.NumericOperation.typ}`
            }
        } else if ("Cast" in instruction){
            return `Cast from ${instruction.Cast.from} to ${instruction.Cast.to}${instruction.Cast.is_signed ? ", keeping sign": ", ignoring sign"}`
        } else if ("Reinterpret" in instruction){
            return `Reinterpret the ${instruction.Reinterpret.is_64 ? "64-bit" : "32-bit"} ${instruction.Reinterpret.is_int_to_float ? "int as a float" : "float as an int"}`
        }
    }
    return JSON.stringify(instruction);
}

export type Error = {
    message: string,
}

function unimplementedError(): Error{
    return {message: "Unimplemented"}
}

function emptyStackError(): Error{
    return {message: "Nothing is on the stack"}
}

// From https://stackoverflow.com/questions/175739/how-can-i-check-if-a-string-is-a-valid-number
function isNumeric(value: string): boolean{
    return !isNaN(parseInt(value)) && !isNaN(parseFloat(value))
}

export function eval_instruction_single_step(instruction: command.SerializedInstruction, stack: (bigint | number)[], memory: {[key:string]: Uint8Array}, globals: {indexToName: string[], names: {[key:string]: (bigint|number)} }, locals: {indexToName: string[], names: {[key:string]: (bigint|number)}}): "Continue" | "Break" | Error{
    if(typeof instruction === "string"){
        switch (instruction) {
            case "Unreachable":
                return {message: "Reached Unreachable"}
            case "Nop":
                break;
            case "Drop":
                stack.pop();
                break;
            case "Return":
                return "Break"
            default:
                return {message: `Unknown instruction: ${instruction}`}
    }
}
    else {
        if("Const" in instruction){
            stack.push(combine_bytes(instruction.Const.lower32bits, instruction.Const.upper32bits));
        }
        else if ("Get" in instruction){
            const env = instruction.Get.is_local ? locals : globals;
            const value = isNumeric(instruction.Get.loc) ? env.names[env.indexToName[parseInt(instruction.Get.loc)]] : env.names[instruction.Get.loc];
            if(typeof value === "undefined"){
                return {message: `${instruction.Get.is_local ? "Local" : "Global"} variable ${instruction.Get.loc} not found`}
            }
            stack.push(value);
            }
        else if ("Set" in instruction){
            const value = stack.pop();
            if(value){
                const env = instruction.Set.is_local ? locals : globals;
                if (isNumeric(instruction.Set.loc) && env.names[env.indexToName[parseInt(instruction.Set.loc)]] !== undefined){
                    env.names[env.indexToName[parseInt(instruction.Set.loc)]] = value;
                }
                else if (env.names[instruction.Set.loc] !== undefined) {
                    env.names[instruction.Set.loc] = value;
                }
                else {
                    return {message: `${instruction.Set.is_local ? "Local" : "Global"} variable ${instruction.Set.loc} not found`}
                }
            }
            else {
                return emptyStackError();
            }
        }
        else if ("Tee" in instruction){
            const value = stack.pop();
            if(value){
                if (isNumeric(instruction.Tee.loc) && locals.names[locals.indexToName[parseInt(instruction.Tee.loc)]] !== undefined){
                    locals.names[locals.indexToName[parseInt(instruction.Tee.loc)]] = value;
                }
                else if (locals.names[instruction.Tee.loc] !== undefined) {
                    locals.names[instruction.Tee.loc] = value;
                }
                else {
                    return {message: `Local variable ${instruction.Tee.loc} not found`}
                }
                stack.push(value);
            }
            else {
                return emptyStackError();
            }
        }
        else if("Load" in instruction){
            // const load = instruction.Load;
            return unimplementedError();
        }
        else if("Store" in instruction){
            // const store = instruction.Store;
            return unimplementedError();
        }
        else if("Memory" in instruction){
            // const memory = instruction.Memory;
            return unimplementedError();
        }
        else if("CountBits" in instruction){
            // const count = instruction.CountBits;
            return unimplementedError();
        }
        else if("NumericOperation" in instruction){
            const op = instruction.NumericOperation.op;
        if("Comparison" in op){
            switch (op.Comparison){
                case "EqualZero":{
                    const val = stack.pop();
                    if(val){
                        stack.push(val === 0 ? 1 : 0)
                    }
                    else {
                        return emptyStackError();
                    }
                    break;
                }

                default:
                    break;
            }
        } else if ("Arithmetic" in op){
            switch (op.Arithmetic){
                case "Addition":
                    break;
                default:
                    break;
            }
        } else if ("Bitwise" in op){
            switch (op.Bitwise){
                case "And":
                    break;
                default:
                    break;
            }
        } else if ("Float" in op){
            switch (op.Float){
                case "AbsoluteValue":
                    break;
                default:
                    break;
            }
        }
        else {
            return {message:"Unknown operation"}
        }

        }
        else if("Cast" in instruction){
            // const cast = instruction.Cast;
            return unimplementedError();
        }
        else if("Reinterpret" in instruction){
            return unimplementedError();
        }
        else if("ControlFlow" in instruction){
            // const cf = instruction.ControlFlow;
            return unimplementedError();
        }
        else {
            return {message: `Unknown instruction: ${JSON.stringify(instruction)}`}
        }
    }
    return "Continue";
}


//string[] | Error
/**
 * Evaluation function generator of a list of instructions, yielding at every step
 * @param instructions List of instructions to evaluate
 * @param env The full environmnent (including globals and parameters)
 * @param localNames Names of local variables (initialized to default values)
 * @returns A list of results or an error with message
 */
export function* eval_instructions(instructions: command.SerializedInstruction[], memory: {[key:string]: Uint8Array}, globals: {indexToName: string[], names: {[key:string]: (bigint|number)} }, locals: {indexToName: string[], names: {[key:string]: (bigint|number)}}){    
    const stack: (bigint|number)[] = [];
    for (const instruction of instructions) {
        const res = eval_instruction_single_step(instruction, stack, memory, globals, locals);
        if (typeof res === "object"){
            return res;
        }
        else if (res === "Break"){
            break;
        }
        yield {instructions, res, stack, memory, globals, locals}
    }
    return stack.map((v) => v.toString());
}