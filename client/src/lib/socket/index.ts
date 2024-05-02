import { STATE } from "../../state";

export async function connectToSocket(uri: string) {
    const { toast } = await import("../toast");

    let retryTime = 1;

    function connect() {
        const ws = new WebSocket(uri);

        ws.binaryType = "arraybuffer";

        ws.addEventListener("open", () => {
            retryTime = 1;
            console.log("Connected to WebSocket server");

            STATE.socket.connectionState = "connected";
            STATE.socket.ws = ws;

            ws.send("Hello, server!");
        });

        ws.addEventListener("message", (evt) => {
            console.log(evt);
        });

        ws.addEventListener("close", () => {
            STATE.socket.connectionState = "disconnected";
            STATE.socket.ws = null;
        });

        ws.addEventListener("error", () => {
            ws.close();

            const timeout = retryTime;

            toast.error(`Failed to connect to server. Retrying in ${timeout}s`);

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
