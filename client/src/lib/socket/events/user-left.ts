import { STATE } from "../../../state";
import { playerLeft } from "../../dom/list-of-players";
import { parseData } from "../../parser";
import { toast } from "../../toast";

export async function handleUserLeft(data: Array<number>) {
    const userId = parseData(data);
    const userIdx = STATE.usersInRoom.findIndex((user) => user.id === userId);

    if (userIdx === -1) {
        return;
    }

    const user = STATE.usersInRoom.splice(userIdx, 1)[0];

    playerLeft(user.id);
    toast(`${user.displayName} left the room`);
}
