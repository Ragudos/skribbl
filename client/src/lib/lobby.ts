import { wsHost, wsProtocol } from "../consts";
import { STATE } from "../state";
import { updateListOfPlayers } from "./dom/list-of-players";
import { showRoomLink } from "./dom/room-link";
import { showWaitingRoom } from "./dom/waiting-room";
import { connectToSocket } from "./socket";
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

    const lobbyForm = getLobbyForm();
    const formData = new FormData(lobbyForm);

    formData.set("mode", mode);

    try {
        STATE.socket.connectionState = "connecting";

        async function connect() {
            const handshakePayload = await (
                await import("./socket/handshake")
            ).getHandshakePayload(formData);

            if (!handshakePayload) {
                return;
            }

            STATE.user = handshakePayload.user;
            STATE.room = handshakePayload.room;
            STATE.binaryProtocolVersion =
                handshakePayload.binaryProtocolVersion;
            STATE.usersInRoom = handshakePayload.usersInRoom;

            await connectToSocket(
                `${wsProtocol}://${wsHost}/ws?sid=${handshakePayload.user.id}`,
            );
            showWaitingRoom();
            updateListOfPlayers();
            showRoomLink();
        }

        await toast.promise(
            "Connecting to server...",
            "Welcome to Skribbl!",
            processError,
            connect,
        );
    } catch (_) {
        STATE.socket.connectionState = "disconnected";
        STATE.socket.ws = null;
        STATE.user = null;
        STATE.room = null;
        STATE.usersInRoom = [];
        STATE.binaryProtocolVersion = null;
    } finally {
        getPlayBtn().disabled = false;
        getCreatePrivateRoomBtn().disabled = false;
    }
}
