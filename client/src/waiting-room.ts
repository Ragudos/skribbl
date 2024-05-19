import { toast } from "./lib/toast";
import { HTMLElementListener } from "./listener";
import { STATE } from "./state";
import { ClientToServerEvents } from "./types";

export const startGameBtnListener = new HTMLElementListener(
    "start-game-btn",
    "click",
    startGame,
);

function startGame() {
    if (
        STATE.binaryProtocolVersion === null ||
        STATE.socket.connectionState !== "connected" ||
        !STATE.room ||
        !STATE.user ||
        STATE.usersInRoom.length === 0 ||
        STATE.room.state !== "waiting"
    ) {
        return;
    }

    if (STATE.room.hostId !== STATE.user.id) {
        toast.error("Only the host can start the game.");

        return;
    }

    STATE.socket.ws.send(
        new Uint8Array([
            STATE.binaryProtocolVersion,
            ClientToServerEvents.StartGame,
        ]),
    );
}
