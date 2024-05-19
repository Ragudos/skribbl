import {
    addUserToListOfPlayersElement,
    getListOfPlayersElement,
    initializeWaitingRoom,
    removeUserFromListOfPlayersElement,
    setClientAsHostIfTrue,
} from "./dom";
import { toast } from "./lib/toast";
import { STATE } from "./state";
import {
    parseObjAsRoomObj,
    parseObjAsUserObj,
    parsePartOfBinaryData,
} from "./utils";

export function handleError(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    toast.error(parsePartOfBinaryData(data, "string"));
}

export function handleConnectError(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    toast.error(parsePartOfBinaryData(data, "string"));
    STATE.socket.ws.close();
}

export function handleUserJoined(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    const userString = parsePartOfBinaryData(data, "string");
    const user = JSON.parse(userString);

    if (!parseObjAsUserObj(user)) {
        throw new Error("Received invalid payload");
    }

    STATE.usersInRoom.push(user);
    addUserToListOfPlayersElement(user);
    getListOfPlayersElement().setAttribute(
        "data-player-count",
        STATE.usersInRoom.length.toString(),
    );
	toast(`${user.displayName} has joined the room.`);
}

export function handleUserLeft(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    const userId = parsePartOfBinaryData(data, "string");
    const userIdx = STATE.usersInRoom.findIndex((user) => {
        return user.id === userId;
    });
	const user = STATE.usersInRoom[userIdx];

    if (userIdx === -1) {
        console.error(
            "Received `UserLeft` event with a non-existent user id as its payload",
        );

        return;
    }

    STATE.usersInRoom.splice(userIdx, 1);
    removeUserFromListOfPlayersElement(userId);
    getListOfPlayersElement().setAttribute(
        "data-player-count",
        STATE.usersInRoom.length.toString(),
    );
	toast(`${user.displayName} has left the room.`);
}

export function handleStartGame(data: Array<number>) {}
export function handlePickAWord(data: Array<number>) {}
export function handleEndGame(data: Array<number>) {}
export function handleResetRoom(data: Array<number>) {}
export function handleNewTurn(data: Array<number>) {}
export function handleNewWord(data: Array<number>) {}
export function handleNewRound(data: Array<number>) {}

export function handleNewHost(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room) {
        throw new Error(
            "Received event `newHost` despite room state being empty.",
        );
    }

    const hostId = parsePartOfBinaryData(data, "string");

    STATE.room.hostId = hostId;

    if (setClientAsHostIfTrue()) {
        toast("You are now the host of this room.");
    }
}

export function handlePointerDown(data: Array<number>) {}
export function handlePointerMove(data: Array<number>) {}
export function handlePointerUp(data: Array<number>) {}
export function handlePointerLeave(data: Array<number>) {}
export function handleChangeColor(data: Array<number>) {}

export function handleSendGameState(data: Array<number>) {
    if (STATE.room || STATE.user || STATE.usersInRoom.length !== 0) {
        throw new Error(
            "Received one whole game state despite state not being empty.",
        );
    }

    const roomString = parsePartOfBinaryData(data, "string");
    const userString = parsePartOfBinaryData(data, "string");
    const usersInRoomString = parsePartOfBinaryData(data, "string");

    const room = JSON.parse(roomString);
    const user = JSON.parse(userString);
    const usersInRoom = JSON.parse(usersInRoomString);

    if (
        !parseObjAsRoomObj(room) ||
        !parseObjAsUserObj(user) ||
        !Array.isArray(usersInRoom) ||
        !usersInRoom.some((user) => parseObjAsUserObj(user))
    ) {
        throw new Error("Received invalid payload from server");
    }

    STATE.room = room;
    STATE.user = user;
    STATE.usersInRoom = usersInRoom;

    initializeWaitingRoom();
}

export function handleSendMessage(data: Array<number>) {}
