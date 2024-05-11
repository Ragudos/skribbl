import { STATE } from "../../../state";
import { parsePartOfBinaryArray } from "../../parser";

export function handleSendUserInfo(data: Array<number>) {
    const stringifiedUser = parsePartOfBinaryArray(data, "string");
    const user = JSON.parse(stringifiedUser);

    if (!("id" in user) || !("displayName" in user)) {
        throw new Error("Received invalid payload");
    }

    STATE.user = user;

    if (user.id === STATE.room?.hostId) {
        document.body.dataset.host = "true";
    }
}
