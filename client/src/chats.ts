import { toast } from "./lib/toast";
import { HTMLElementListener } from "./listener";
import { STATE } from "./state";
import { ClientToServerEvents } from "./types";
import { turnNumberToArrayOfU8Int } from "./utils";

export const chatFormListener = new HTMLElementListener(
    "chat-form",
    "submit",
    onChatSubmit,
);

export function onChatSubmit(e: SubmitEvent) {
    e.preventDefault();

    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (
        STATE.binaryProtocolVersion === null ||
        !STATE.room ||
        !STATE.user ||
        STATE.usersInRoom.length === 0
    ) {
        return;
    }

    const target = e.currentTarget as HTMLFormElement;
    const formData = new FormData(target);
    const chatMessage = formData.get("chat");

    if (!chatMessage) {
        toast.error("Please type in a message");

        return;
    }

    const chatMessageBinary = new TextEncoder().encode(chatMessage.toString());
    const chatMessageLengthIndicator = turnNumberToArrayOfU8Int(
        chatMessageBinary.length,
    );

    STATE.socket.ws.send(
        new Uint8Array([
            STATE.binaryProtocolVersion,
            ClientToServerEvents.Message,
            chatMessageLengthIndicator.length,
            ...chatMessageLengthIndicator,
            ...chatMessageBinary,
        ]),
    );
}
