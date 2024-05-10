import { STATE } from "../../../state";
import { addUserToDOM } from "../../dom/list-of-players";
import { parseData } from "../../parser";
import { toast } from "../../toast";

export async function handleUserJoined(data: Array<number>) {
    try {
        const user = parseData(data, "string");
        const userObj = JSON.parse(user);

        if (!("id" in userObj) || !("displayName" in userObj)) {
            throw new Error("Invalid user data");
        }

        STATE.usersInRoom.push(userObj);
        addUserToDOM(userObj);
        toast(`${userObj.displayName} joined the room`);
    } catch (err) {
        console.error(err);
    }
}
