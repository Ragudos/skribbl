import { STATE } from "../../../state";
import { addUserToDOM } from "../../dom/list-of-players";
import { parsePartOfBinaryArray } from "../../parser";
import { toast } from "../../toast";

export async function handleUserJoined(data: Array<number>) {
    const user = parsePartOfBinaryArray(data, "string");
    const userObj = JSON.parse(user);

    if (!("id" in userObj) || !("displayName" in userObj)) {
        throw new Error("Invalid user data");
    }

    STATE.usersInRoom.push(userObj);
    addUserToDOM(userObj);
    toast(`${userObj.displayName} joined the room`);
}
