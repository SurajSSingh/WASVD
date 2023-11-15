// place files you want to import through the `$lib` alias in this folder.
import type * as command from "$lib/bindings"

export function combine_bytes(lower: number, upper:number): bigint{
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
        }
    }
    return JSON.stringify(instruction);
}