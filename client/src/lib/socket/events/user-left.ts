import { STATE } from "../../../state";
import { playerLeft } from "../../dom/list-of-players";
import { parseData } from "../../parser";
import { toast } from "../../toast";

export async function handleUserLeft(data: Array<number>) {
    try {
        const userId = parseData(data, "string");
        const userIdx = STATE.usersInRoom.findIndex(
            (user) => user.id === userId,
        );

        if (userIdx === -1) {
            return;
        }

        const user = STATE.usersInRoom.splice(userIdx, 1)[0];

        playerLeft(user.id);
        toast(`${user.displayName} left the room`);
    } catch (err) {
        console.error(err);
    }
}
