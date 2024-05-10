import { STATE } from "../../../state";
import { parseData } from "../../parser";

export function handleSendUserInfo(data: Array<number>) {
    try {
        const stringifiedUser = parseData(data, "string");
        const user = JSON.parse(stringifiedUser);

        if (!("id" in user) || !("displayName" in user)) {
            throw new Error("Received invalid payload");
        }

        STATE.user = user;
    } catch (err) {
        console.error(err);
    }
}
