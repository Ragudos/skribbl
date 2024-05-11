import { STATE } from "../../../state";
import { parsePartOfBinaryArray } from "../../parser";
import { toast } from "../../toast";

export function handleError(data: Array<number>) {
    try {
        toast.error(parsePartOfBinaryArray(data, "string"));
    } catch (err) {
        console.error(err);
    }
}

export function handleConnectError(data: Array<number>) {
    try {
        toast.error(parsePartOfBinaryArray(data, "string"));

        STATE.socket.ws?.close();
    } catch (err) {
        console.error(err);
    }
}
