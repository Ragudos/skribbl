import {
    MAX_DISPLAY_NAME_LENGTH,
    MIN_DISPLAY_NAME_LENGTH,
    wsHost,
    wsProtocol,
} from "./consts";
import {
    getCreatePrivateRoomBtn,
    getDisplayName,
    getLobbyForm,
    getPlayBtn,
} from "./lib/lobby";
import { STATE } from "./state";
import "./styles/index.css";

const playBtn = getPlayBtn();
const createPrivateRoomBtn = getCreatePrivateRoomBtn();

playBtn.addEventListener("click", async () => {
    const lobbyForm = getLobbyForm();
    const formData = new FormData(lobbyForm);
    const displayName = formData.get("displayName") as string;
    const { toast } = await import("./lib/toast");

    if (!displayName) {
        return toast.error("Display name cannot be empty");
    } else if (
        displayName.length < MIN_DISPLAY_NAME_LENGTH ||
        displayName.length > MAX_DISPLAY_NAME_LENGTH
    ) {
        return toast.error("Display name must be between 3 and 20 characters");
    }

    try {
        STATE.socket.connectionState = "connecting";

        const [_toastId, handshakePayload] = await toast.promise(
            "Connecting to server...",
            "Welcome to Skribbl!",
            (err) => {
                if (err instanceof Error) {
                    return err.message;
                }

                if (typeof err === "string") {
                    return err;
                }

                return "An unknown error occurred";
            },
            async () => {
                const { getHandshakePayload } = await import(
                    "./lib/socket/handshake"
                );

                return getHandshakePayload(formData);
            },
        );

        if (!handshakePayload) {
            return;
        }

        const userId = handshakePayload.user.id;
        const { connectToSocket } = await import("./lib/socket");

        await connectToSocket(`${wsProtocol}://${wsHost}/ws?sid=${userId}`);
    } catch (_) {
        STATE.socket.connectionState = "disconnected";
    }
});

createPrivateRoomBtn.addEventListener("click", async () => {
    const displayName = getDisplayName();
    const { toast } = await import("./lib/toast");

    if (!displayName) {
        return toast.error("Display name cannot be empty");
    } else if (
        displayName.length < MIN_DISPLAY_NAME_LENGTH ||
        displayName.length > MAX_DISPLAY_NAME_LENGTH
    ) {
        return toast.error("Display name must be between 3 and 20 characters");
    }
});
