import { wsHost, wsProtocol } from "../consts";
import { STATE } from "../state";
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
    const lobbyForm = getLobbyForm();
    const formData = new FormData(lobbyForm);
    const { toast } = await import("./toast");

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

            await (
                await import("./socket")
            ).connectToSocket(
                `${wsProtocol}://${wsHost}/ws?sid=${handshakePayload.user.id}`,
            );
            (await import("./dom/list-of-players")).updateListOfPlayers();
            (await import("./dom/waiting-room")).showWaitingRoom();
            (await import("./dom/room-link")).showRoomLink();
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
    }
}
