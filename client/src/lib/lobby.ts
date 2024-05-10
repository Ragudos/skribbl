import { wsHost, wsProtocol } from "../consts";
import { STATE } from "../state";
import { connect } from "./socket";
import { toast } from "./toast";
import { processError } from "./utils";

export function getPlayBtn() {
    return document.getElementById("play-btn") as HTMLButtonElement;
}

export function getCreatePrivateRoomBtn() {
    return document.getElementById(
        "create-private-room-btn",
    ) as HTMLButtonElement;
}

export function getLobbyForm() {
    return document.getElementById("lobby-form") as HTMLFormElement;
}

export function getDisplayName() {
    return (document.getElementById("display-name") as HTMLInputElement).value;
}

export function getRoomId() {
    return (document.getElementById("room-id") as HTMLInputElement).value;
}

export async function submit(mode: "play" | "create") {
    if (STATE.socket.connectionState === "connecting") {
        return;
    }

    try {
        const displayName = getDisplayName();
        const roomId = getRoomId();

        await toast.promise(
            "Connecting to server...",
            "Welcome to Skribbl!",
            processError,
            connect(
                `${wsProtocol}://${wsHost}/ws?displayName=${displayName}&roomId=${roomId}&mode=${mode}`,
            ),
        );
    } catch (_) {
        STATE.socket.connectionState = "disconnected";
        STATE.socket.ws = null;
        STATE.user = null;
        STATE.room = null;
        STATE.usersInRoom = [];
        STATE.binaryProtocolVersion = null;
    }
}
