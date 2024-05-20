import { wsHost, wsProtocol } from "./consts";
import { initializeWaitingRoom } from "./dom";
import { toast } from "./lib/toast";
import { HTMLElementListener, WebSocketListener } from "./listener";
import { connect, wsOnClose, wsOnError, wsOnMessage } from "./socket";
import { STATE } from "./state";
import { processError } from "./utils";
import { startGameBtnListener } from "./waiting-room";

export const playFormListener = new HTMLElementListener(
    "lobby-form",
    "submit",
    onPlayFormSubmit,
);
export const playBtnListener = new HTMLElementListener(
    "play-btn",
    "click",
    onPlayBtnClick,
);
export const createPrivateRoomBtnListener = new HTMLElementListener(
    "create-private-room-btn",
    "click",
    onCreatePrivateRoomBtnClick,
);

function onPlayFormSubmit(e: Event) {
    e.preventDefault();
    connectToSocket("play");
}

function onPlayBtnClick(_e: Event) {
    connectToSocket("play");
}

function onCreatePrivateRoomBtnClick(_e: Event) {
    connectToSocket("create");
}

async function connectToSocket(mode: "play" | "create") {
    if (STATE.socket.connectionState === "connecting") {
        return;
    }

    togglePlayCtrls(true);

    const formData = new FormData(getPlayForm());
    const displayName = formData.get("displayName")?.toString();
    const roomId = formData.get("roomId")?.toString();

    try {
        const [_, ws] = await toast.promise(
            "Loading...",
            "Welcome to Skribbl",
            processError,
            async () => {
                const binaryProtocolVersion = await (
                    await fetch("/ws/binary-protocol-version")
                ).json();

                STATE.binaryProtocolVersion = binaryProtocolVersion;

                const ws = await connect(
                    `${wsProtocol}://${wsHost}/ws?displayName=${displayName}&roomId=${roomId}&mode=${mode}`,
                );

                return ws;
            },
        );

        const onmessage = new WebSocketListener(ws, "message", wsOnMessage);
        const onclose = new WebSocketListener(ws, "close", wsOnClose);
        const onerror = new WebSocketListener(ws, "error", wsOnError);

        onmessage.listen();
        onclose.listen();
        onerror.listen();

        STATE.socket = {
            connectionState: "connected",
            ws,
            listeners: {
                onmessage,
                onclose,
                onerror,
            },
        };

        playFormListener.disconnect();
        playBtnListener.disconnect();
        createPrivateRoomBtnListener.disconnect();
        startGameBtnListener.listen();
        initializeWaitingRoom();
    } catch (_) {
        STATE.binaryProtocolVersion = null;

        if (STATE.socket.connectionState === "connected") {
            STATE.socket.ws.close();
        }
    } finally {
        togglePlayCtrls(false);
    }
}

function togglePlayCtrls(isDisabled: boolean) {
    const playBtn = document.getElementById("play-btn");
    const createPrivateRoomBtn = document.getElementById(
        "create-private-room-btn",
    );

    if (
        !playBtn ||
        !createPrivateRoomBtn ||
        !(playBtn instanceof HTMLButtonElement) ||
        !(createPrivateRoomBtn instanceof HTMLButtonElement)
    ) {
        throw new Error(
            "Buttons with ids: ['play-btn', 'create-private-room-btn'] can't be found",
        );
    }

    playBtn.disabled = isDisabled;
    createPrivateRoomBtn.disabled = isDisabled;
}

function getPlayForm() {
    const playForm = document.getElementById("lobby-form");

    if (!playForm || !(playForm instanceof HTMLFormElement)) {
        throw new Error("Cannot find form in lobby with id: lobby-form");
    }

    return playForm;
}
