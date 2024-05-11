import { STATE } from "../../../state";
import { updateHost } from "../../dom/list-of-players";
import { parsePartOfBinaryArray } from "../../parser";
import { toast } from "../../toast";

export function handleNewHost(data: Array<number>) {
    const newHostId = parsePartOfBinaryArray(data, "string");

    updateHost(newHostId);
    STATE.room!.hostId = newHostId;

    if (STATE.user?.id !== newHostId) {
        return;
    }

    toast("You are now the host of this room.");
}
