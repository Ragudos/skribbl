import {
    canvasPointerDownListener,
    canvasPointerLeaveListener,
    canvasPointerMoveListener,
    windowPointerUpListenerForCanvas,
} from "./canvas";
import { chatFormListener } from "./chats";
import {
    clearListOfPlayers,
    getChatContainer,
    getListOfChatsContainer,
    getListOfPlayersElement,
    getRoomLinkElement,
    getRoomLinkInputElement,
    getUserToDrawUsername,
    getWordList,
    resetBodyAttributes,
    showRoom,
    togglePickingAWordModal,
} from "./dom";
import {
    handleAddScore,
    handleChangeColor,
    handleConnectError,
    handleEndGame,
    handleError,
    handleNewHost,
    handleNewRound,
    handleNewTurn,
    handleNewWord,
    handlePickAWord,
    handlePointerDown,
    handlePointerLeave,
    handlePointerMove,
    handlePointerUp,
    handleResetRoom,
    handleRevealWord,
    handleSendGameState,
    handleSendMessage,
    handleStartGame,
    handleSystemMessage,
    handleTick,
    handleUserGuessed,
    handleUserJoined,
    handleUserLeft,
} from "./events";
import {
    createPrivateRoomBtnListener,
    playBtnListener,
    playFormListener,
} from "./lobby";
import { STATE } from "./state";
import { ServerToClientEvents } from "./types";
import { parsePartOfBinaryData } from "./utils";

export function connect(uri: string) {
    return new Promise<WebSocket>((resolve, reject) => {
        const ws = new WebSocket(uri);

        ws.binaryType = "arraybuffer";

        function onConnect() {
            ws.removeEventListener("open", onConnect);
            ws.removeEventListener("error", onError);
            ws.addEventListener("message", onMessage);
        }

        function onError() {
            ws.removeEventListener("open", onConnect);
            ws.removeEventListener("error", onError);
            ws.close();
            reject();
        }

        function onMessage(e: MessageEvent) {
            ws.removeEventListener("message", onMessage);

            if (!(e.data instanceof ArrayBuffer)) {
                reject("Received non-binary data from server.");
                return;
            }

            const data = Array.from(new Uint8Array(e.data));

            if (data.splice(0, 1)[0] !== STATE.binaryProtocolVersion) {
                ws.close();
                reject("Binary protocol version mismatch");
                return;
            }

            const eventType = data.splice(0, 1)[0];

            try {
                switch (eventType) {
                    case ServerToClientEvents.ConnectError:
                        {
                            ws.close();
                            reject(parsePartOfBinaryData(data, "string"));
                        }
                        break;
                    case ServerToClientEvents.SendGameState:
                        {
                            handleSendGameState(data);
                            resolve(ws);
                        }
                        break;
                }
            } catch (err) {
                ws.close();
                reject(err);
            }
        }

        ws.addEventListener("open", onConnect);
        ws.addEventListener("error", onError);
    });
}

export function wsOnMessage(e: MessageEvent) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!(e.data instanceof ArrayBuffer)) {
        console.error("Received non-binary data from server.");
        return;
    }

    const data = Array.from(new Uint8Array(e.data));

    if (data.splice(0, 1)[0] !== STATE.binaryProtocolVersion) {
        return;
    }

    const eventType = data.splice(0, 1)[0];

    try {
        switch (eventType) {
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
            case ServerToClientEvents.NewRound:
                handleNewRound(data);
                break;
            case ServerToClientEvents.NewHost:
                handleNewHost(data);
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
            case ServerToClientEvents.PointerLeave:
                handlePointerLeave(data);
                break;
            case ServerToClientEvents.ChangeColor:
                handleChangeColor(data);
                break;
            case ServerToClientEvents.SendGameState:
                handleSendGameState(data);
                break;
            case ServerToClientEvents.SendMessage:
                handleSendMessage(data);
                break;
            case ServerToClientEvents.AddScore:
                handleAddScore(data);
                break;
            case ServerToClientEvents.Tick:
                handleTick(data);
                break;
            case ServerToClientEvents.UserGuessed:
                handleUserGuessed(data);
                break;
            case ServerToClientEvents.SystemMessage:
                handleSystemMessage(data);
                break;
            case ServerToClientEvents.RevealWord:
                handleRevealWord(data);
                break;
            default:
                throw new Error("Received invalid event from server.");
        }
    } catch (err) {
        console.error(err);
        STATE.socket.ws.close();
    }
}

export function wsOnClose() {
    if (STATE.socket.connectionState !== "connected") {
        throw new Error(
            "Called `wsOnClose` despite `STATE.socket.connectionState` not being `connected`",
        );
    }

    STATE.socket.listeners.onerror.disconnect();
    STATE.socket.listeners.onclose.disconnect();
    STATE.socket.listeners.onmessage.disconnect();

    STATE.socket = {
        connectionState: "disconnected",
        ws: null,
    };

    if (
        STATE.room &&
        STATE.room.state !== "waiting" &&
        STATE.room.state !== "finished"
    ) {
        canvasPointerDownListener.disconnect();
        canvasPointerLeaveListener.disconnect();
        windowPointerUpListenerForCanvas.disconnect();
        canvasPointerMoveListener.disconnect();
    }

    STATE.room = null;
    STATE.user = null;
    STATE.usersInRoom = [];

    STATE.canvas?.destroy();
    STATE.canvas = null;

    STATE.binaryProtocolVersion = null;

    if (STATE.wordListBtnListeners) {
        for (let i = 0; i < STATE.wordListBtnListeners.length; ++i) {
            STATE.wordListBtnListeners[i].disconnect();
        }

        STATE.wordListBtnListeners = null;

        getWordList().innerHTML = "";
    }

    getUserToDrawUsername().textContent = "";
    togglePickingAWordModal(false);

    getRoomLinkElement().setAttribute("hidden", "");
    getListOfPlayersElement().setAttribute("hidden", "");

    resetBodyAttributes();
    getRoomLinkInputElement().value = "";
    clearListOfPlayers();

    getChatContainer().setAttribute("hidden", "");
    getListOfChatsContainer().innerHTML = "";

    chatFormListener.disconnect();

    showRoom("lobby");

    playBtnListener.listen();
    playFormListener.listen();
    createPrivateRoomBtnListener.listen();
}

export function wsOnError() {}
