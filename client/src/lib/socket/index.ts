import { STATE } from "../../state";
import { WebSocketEvents } from "../../types";
import { toast } from "../toast";

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
                    (await import("./events/error")).handleError(data);
                    break;
                case WebSocketEvents.UserJoined:
                    (await import("./events/user-joined")).handleUserJoined(
                        data,
                    );
                    break;
                case WebSocketEvents.UserLeft:
                    (await import("./events/user-left")).handleUserLeft(data);
                    break;
                case WebSocketEvents.StartGame:
                    (await import("./events/start-game")).handleStartGame(data);
                    break;
                case WebSocketEvents.EndGame:
                    (await import("./events/end-game")).handleEndGame(data);
                    break;
                case WebSocketEvents.NewRound:
                    (await import("./events/new-round")).handleNewRound(data);
                    break;
                case WebSocketEvents.NewUserToDraw:
                    (
                        await import("./events/new-user-to-draw")
                    ).handleNewUserToDraw(data);
                    break;
                case WebSocketEvents.PointerDown:
                    (await import("./events/pointer-down")).handlePointerDown(
                        data,
                    );
                    break;
                case WebSocketEvents.PointerMove:
                    (await import("./events/pointer-move")).handlePointerMove(
                        data,
                    );
                    break;
                case WebSocketEvents.PointerUp:
                    (await import("./events/pointer-up")).handlePointerUp(data);
                    break;
                case WebSocketEvents.ChangeColor:
                    (await import("./events/change-color")).handleChangeColor(
                        data,
                    );
                    break;
                case WebSocketEvents.Tick:
                    (await import("./events/tick")).handleTick(data);
                    break;
                case WebSocketEvents.ResetRoom:
                    (await import("./events/reset-room")).handleResetRoom(data);
                    break;
                case WebSocketEvents.NewHost:
                    (await import("./events/new-host")).handleNewHost(data);
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
