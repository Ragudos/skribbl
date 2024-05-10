import { STATE } from "../../../state";
import { parseData } from "../../parser";

export function handleSendRoomInfo(data: Array<number>) {
    try {
        const roomInfo = parseData(data, "string");
        const roomInfoObj = JSON.parse(roomInfo);

        if (
            !("id" in roomInfoObj) ||
            !("hostId" in roomInfoObj) ||
            !("visibility" in roomInfoObj) ||
            !("state" in roomInfoObj) ||
            !("maxUsers" in roomInfoObj) ||
            !("maxRounds" in roomInfoObj)
        ) {
            throw new Error("Received invalid payload");
        }

        STATE.room = roomInfoObj;
    } catch (err) {
        console.error(err);
    }
}
