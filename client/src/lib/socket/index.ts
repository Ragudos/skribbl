import { STATE } from "../../state";
import { WebSocketEvents } from "../../types";
import { toast } from "../toast";
import { handleChangeColor } from "./events/change-color";
import { handleEndGame } from "./events/end-game";
import { handleError } from "./events/error";
import { handleNewHost } from "./events/new-host";
import { handleNewRound } from "./events/new-round";
import { handleNewUserToDraw } from "./events/new-user-to-draw";
import { handlePointerDown } from "./events/pointer-down";
import { handlePointerMove } from "./events/pointer-move";
import { handlePointerUp } from "./events/pointer-up";
import { handleResetRoom } from "./events/reset-room";
import { handleStartGame } from "./events/start-game";
import { handleTick } from "./events/tick";
import { handleUserJoined } from "./events/user-joined";
import { handleUserLeft } from "./events/user-left";

export async function connectToSocket(uri: string) {
    let retryTime = 1;

    function connect() {
        STATE.socket.connectionState = "connecting";

        const ws = new WebSocket(uri);

        ws.binaryType = "arraybuffer";

        ws.addEventListener("open", () => {
            retryTime = 1;
            console.log("Connected to WebSocket server");

            STATE.socket.connectionState = "connected";
            STATE.socket.ws = ws;
        });

        ws.addEventListener("message", async (evt) => {
            if (!(evt.data instanceof ArrayBuffer)) {
                console.error("Received non-arraybuffer data from server");
                return;
            }

            const data = Array.from(new Uint8Array(evt.data));

            if (data.length < 2) {
                console.error("Received invalid data from server");
                return;
            }

            const version = data.splice(0, 1)[0];

            if (version !== STATE.binaryProtocolVersion) {
                console.error("Received data with invalid protocol version");
                return;
            }

            const event = data.splice(0, 1)[0];

            switch (event) {
                case WebSocketEvents.Error:
                    handleError(data);
                    break;
                case WebSocketEvents.UserJoined:
                    handleUserJoined(data);
                    break;
                case WebSocketEvents.UserLeft:
                    handleUserLeft(data);
                    break;
                case WebSocketEvents.StartGame:
                    handleStartGame(data);
                    break;
                case WebSocketEvents.EndGame:
                    handleEndGame(data);
                    break;
                case WebSocketEvents.NewRound:
                    handleNewRound(data);
                    break;
                case WebSocketEvents.NewUserToDraw:
                    handleNewUserToDraw(data);
                    break;
                case WebSocketEvents.PointerDown:
                    handlePointerDown(data);
                    break;
                case WebSocketEvents.PointerMove:
                    handlePointerMove(data);
                    break;
                case WebSocketEvents.PointerUp:
                    handlePointerUp(data);
                    break;
                case WebSocketEvents.ChangeColor:
                    handleChangeColor(data);
                    break;
                case WebSocketEvents.Tick:
                    handleTick(data);
                    break;
                case WebSocketEvents.ResetRoom:
                    handleResetRoom(data);
                    break;
                case WebSocketEvents.NewHost:
                    handleNewHost(data);
                    break;
                default:
                    console.error("Received unknown event from server");
                    break;
            }
        });

        ws.addEventListener("close", () => {
            STATE.socket.connectionState = "disconnected";
            STATE.socket.ws = null;
        });

        ws.addEventListener("error", (evt) => {
            console.log(evt);
            ws.close();

            const timeout = retryTime;

            toast.error(`An error occured. Reconnecting in ${timeout}s`);

            retryTime = Math.min(retryTime * 2, 64);

            setTimeout(
                connect,
                (() => {
                    return timeout * 1000;
                })(),
            );
        });
    }

    connect();
}
