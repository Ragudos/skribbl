import { STATE } from "../../../state";
import { playerLeft } from "../../dom/list-of-players";
import { parseData } from "../../parser";
import { toast } from "../../toast";

export async function handleUserLeft(data: Array<number>) {
    const userId = parseData(data);

    const user = STATE.usersInRoom.splice(
        STATE.usersInRoom.findIndex((user) => user.id === userId),
        1,
    )[0];

    playerLeft(user.id);
    toast(`${user.displayName} left the room`);
}
