import { STATE } from "../../../state";
import { parseData } from "../../parser";

export function handleSendUsersInRoom(data: Array<number>) {
    try {
        const users = parseData(data, "string");
        const usersObj = JSON.parse(users);

        if (!Array.isArray(usersObj)) {
            throw new Error("Received invalid payload");
        }

        for (let i = 0; i < usersObj.length; ++i) {
            if (!("id" in usersObj[i]) || !("displayName" in usersObj[i])) {
                throw new Error("Received invalid payload");
            }
        }

        STATE.usersInRoom = usersObj;
    } catch (err) {
        console.error(err);
    }
}
