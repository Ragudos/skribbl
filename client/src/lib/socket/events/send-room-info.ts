import { STATE } from "../../../state";
import { showRoomLink } from "../../dom/room-link";
import { showWaitingRoom } from "../../dom/rooms";
import { parsePartOfBinaryArray } from "../../parser";

export function handleSendRoomInfo(data: Array<number>) {
    const roomInfo = parsePartOfBinaryArray(data, "string");
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

    showWaitingRoom();
    showRoomLink();
}
