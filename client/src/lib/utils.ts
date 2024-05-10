import { STATE } from "../state";

export function processError(err: unknown) {
    if (err instanceof Error) {
        return err.message;
    }

    return typeof err === "string" ? err : "An unknown error occurred";
}

export function getVersion(data: Array<number>) {
    const version = data.splice(0, 1)[0];

    if (version !== STATE.binaryProtocolVersion) {
        throw new Error("Binary protocol version mismatch");
    }

    return version;
}
