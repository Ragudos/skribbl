import { STATE } from "../../../state";
import { updateListOfPlayers } from "../../dom/list-of-players";
import { parsePartOfBinaryArray } from "../../parser";

export function handleSendUsersInRoom(data: Array<number>) {
    const users = parsePartOfBinaryArray(data, "string");
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

    updateListOfPlayers();
}
