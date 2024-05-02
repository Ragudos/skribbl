import { HandshakePayload } from "../../types";

function parseHandshakePayload(data: any): data is HandshakePayload {
    if (typeof data !== "object") {
        return false;
    }

    if (typeof data.user !== "object") {
        return false;
    }

    if (typeof data.room !== "object") {
        return false;
    }

    if (typeof data.user.id !== "string") {
        return false;
    }

    if (typeof data.user.displayName !== "string") {
        return false;
    }

    if (typeof data.room.id !== "string") {
        return false;
    }

    if (typeof data.room.hostId !== "string") {
        return false;
    }

    if (
        data.room.visibility !== "public" &&
        data.room.visibility !== "private"
    ) {
        return false;
    }

    if (data.room.state !== "waiting" && data.room.state !== "finished") {
        return false;
    }

    if (typeof data.room.state === "object") {
        if (typeof data.room.state.playing !== "object") {
            return false;
        }

        if (typeof data.room.state.playing.userToDraw !== "string") {
            return false;
        }
    }

    return true;
}

/**
 * @param formData
 * Accepts a FormData object and returns a Promise that resolves to a HandshakePayload object.
 * This does not verify whether the formData is valid or not. This will throw an error
 * if the received {@link HandshakePayload} is invalid, or the `fetch API` throws an error.
 */
export function getHandshakePayload(
    formData: FormData,
): Promise<HandshakePayload | undefined> {
    return new Promise(async (resolve, reject) => {
        try {
            const response = await fetch("/ws/handshake", {
                method: "POST",
                body: formData,
            });

            if (!response.ok) {
                const errorText = await response.text();
                return reject(errorText);
            }

            const data = await response.json();

            if (!parseHandshakePayload(data)) {
                console.error("Invalid handshake payload ", data);
                return reject("Something went wrong. Please try again later.");
            }

            resolve(data);
        } catch (err) {
            reject(err);
        }
    });
}
