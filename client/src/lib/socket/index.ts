import { STATE } from "../../state";
import { ServerToClientEvents } from "../../types";
import { toast } from "../toast";
import { getVersion } from "../utils";
import { handleChangeColor } from "./events/change-color";
import { handleEndGame } from "./events/end-game";
import { handleConnectError, handleError } from "./events/error";
import { handleNewHost } from "./events/new-host";
import { handleNewRound } from "./events/new-round";
import { handleNewTurn } from "./events/new-turn";
import { handleNewWord } from "./events/new-word";
import { handlePickAWord } from "./events/pick-a-word";
import { handlePointerDown } from "./events/pointer-down";
import { handlePointerMove } from "./events/pointer-move";
import { handlePointerUp } from "./events/pointer-up";
import { handleResetRoom } from "./events/reset-room";
import { handleSendRoomInfo } from "./events/send-room-info";
import { handleSendUserInfo } from "./events/send-user-info";
import { handleSendUsersInRoom } from "./events/send-users-in-room";
import { handleStartGame } from "./events/start-game";
import { handleUserJoined } from "./events/user-joined";
import { handleUserLeft } from "./events/user-left";

export function connectToSocket(uri: string) {
    return new Promise((res, rej) => {
        const ws = new WebSocket(uri);

        ws.binaryType = "arraybuffer";
        STATE.socket.connectionState = "connecting";

        ws.addEventListener(
            "open",
            () => {
                STATE.socket.connectionState = "connected";
                STATE.socket.ws = ws;

                res(undefined);
            },
            { once: true },
        );

        ws.addEventListener(
            "error",
            () => {
                STATE.socket.connectionState = "disconnected";
                rej(new Error("Failed to connect to server."));
            },
            { once: true },
        );
    });
}

export async function connect(uri: string) {
    if (STATE.socket.connectionState === "connecting") {
        return;
    }

    const binaryProtocolVersionRes = await fetch("/ws/binary-protocol-version");
    const binaryProtocolVersion = await binaryProtocolVersionRes.json();

    STATE.binaryProtocolVersion = binaryProtocolVersion;

    await connectToSocket(uri);

    if (!STATE.socket.ws) {
        console.error("Reached an unreachable state.");
        return;
    }

    STATE.socket.ws.addEventListener("error", onError);
    STATE.socket.ws.addEventListener("close", onClose);
    STATE.socket.ws.addEventListener("message", onMessage);
}

function onMessage(evt: any) {
    if (!(evt.data instanceof ArrayBuffer)) {
        console.error("Received non-binary data from server.");
        return;
    }

    const data = Array.from(new Uint8Array(evt.data));

    if (data.length < 2) {
        console.error("Received invalid paylaod");
        return;
    }

    const version = getVersion(data),
        event = data.splice(0, 1)[0];

    switch (event) {
        case ServerToClientEvents.Error:
            handleError(data);
            break;
        case ServerToClientEvents.ConnectError:
            handleConnectError(data);
            break;
        case ServerToClientEvents.UserJoined:
            handleUserJoined(data);
            break;
        case ServerToClientEvents.UserLeft:
            handleUserLeft(data);
            break;
        case ServerToClientEvents.StartGame:
            handleStartGame(data);
            break;
        case ServerToClientEvents.PickAWord:
            handlePickAWord(data);
            break;
        case ServerToClientEvents.EndGame:
            handleEndGame(data);
            break;
        case ServerToClientEvents.ResetRoom:
            handleResetRoom(data);
            break;
        case ServerToClientEvents.NewTurn:
            handleNewTurn(data);
            break;
        case ServerToClientEvents.NewWord:
            handleNewWord(data);
            break;
        case ServerToClientEvents.NewHost:
            handleNewHost(data);
            break;
        case ServerToClientEvents.NewRound:
            handleNewRound(data);
            break;
        case ServerToClientEvents.PointerDown:
            handlePointerDown(data);
            break;
        case ServerToClientEvents.PointerMove:
            handlePointerMove(data);
            break;
        case ServerToClientEvents.PointerUp:
            handlePointerUp(data);
            break;
        case ServerToClientEvents.ChangeColor:
            handleChangeColor(data);
            break;
        case ServerToClientEvents.SendUserInfo:
            handleSendUserInfo(data);
            break;
        case ServerToClientEvents.SendRoomInfo:
            handleSendRoomInfo(data);
            break;
        case ServerToClientEvents.SendUsersInRoomInfo:
            handleSendUsersInRoom(data);
            break;
        case ServerToClientEvents.SendMessage:
            console.log("Unimplemented event: SendMessage");
            break;
        default:
            console.error("Received unknown event from server.");
            break;
    }

    console.log(STATE);
}

function onClose() {
    STATE.socket.connectionState = "disconnected";
    STATE.socket.ws = null;

    toast.error("Disconnected from the server.");
}

function onError() {
    STATE.socket.ws?.close();

    toast.error("Something went wrong. Please reconnect to the server.");
}
