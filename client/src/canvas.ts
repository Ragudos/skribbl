import { HTMLElementListener, WindowListener } from "./listener";
import { STATE } from "./state";
import { ClientToServerEvents } from "./types";

export type CanvasDrawMode = "draw" | "erase";

export type Point = {
    x: number;
    y: number;
};

export const canvasPointerDownListener = new HTMLElementListener(
    "drawing-canvas",
    "pointerdown",
    onCanvasPointerDown,
);
export const canvasPointerLeaveListener = new HTMLElementListener(
    "drawing-canvas",
    "pointerleave",
    onCanvasPointerLeave,
);
export const windowPointerMoveListenerForCanvas = new WindowListener(
    "pointermove",
    onWindowPointerMove,
);
export const windowPointerUpListenerForCanvas = new WindowListener(
    "pointerup",
    onWindowPointerUp,
);

export class Canvas {
    static DEFAULT_LINE_WIDTH = 10;
    static DEFAULT_LINE_COLOR = "black";
    static DEFAULT_FILL_STYLE = "white";
    static DEFAULT_LINE_CAP: CanvasLineCap = "round";

    private _prevPoint: null | Point;

    readonly _ctx: CanvasRenderingContext2D;
    private _drawMode: CanvasDrawMode;

    isDrawing: boolean;

    constructor(ctx: CanvasRenderingContext2D) {
        this._ctx = ctx;

        this._prevPoint = null;

        this._drawMode = "draw";

        this.isDrawing = false;

        this._ctx.lineCap = Canvas.DEFAULT_LINE_CAP;
        this._ctx.lineWidth = Canvas.DEFAULT_LINE_WIDTH;
        this._ctx.strokeStyle = Canvas.DEFAULT_LINE_COLOR;
        this._ctx.fillStyle = Canvas.DEFAULT_FILL_STYLE;
    }

    destroy() {
        this._ctx.clearRect(
            0,
            0,
            this._ctx.canvas.width,
            this._ctx.canvas.height,
        );

		canvasPointerDownListener.disconnect();
		windowPointerUpListenerForCanvas.disconnect();
		windowPointerMoveListenerForCanvas.disconnect();
		canvasPointerLeaveListener.disconnect();
    }

    drawLine(x: number, y: number) {
        if (!this.isDrawing) {
            return;
        }

        this._ctx.beginPath();

        switch (this._drawMode) {
            case "erase":
                this._erase(x, y);
                break;
            case "draw":
                this._draw(x, y);
                break;
        }

        this._ctx.stroke();
    }

    changeBackground(color: string) {
        if (
            this._drawMode === "erase" &&
            this._ctx.globalCompositeOperation === "destination-out"
        ) {
            this._ctx.globalCompositeOperation = "source-over";
        }

        this._ctx.fillStyle = color;
        this._ctx.fillRect(
            0,
            0,
            this._ctx.canvas.width,
            this._ctx.canvas.height,
        );

        if (this._drawMode === "erase") {
            this._ctx.globalCompositeOperation = "destination-out";
        }
    }

    resetPrevPoint() {
        this._prevPoint = null;
    }

    get drawMode() {
        return this._drawMode;
    }

    private _erase(x: number, y: number) {
        this._ctx.strokeStyle = "white";
        this._moveLine(x, y);
    }

    private _draw(x: number, y: number) {
        this._moveLine(x, y);
    }

    private _moveLine(x: number, y: number) {
        if (this._prevPoint === null) {
            this._ctx.moveTo(x, y);
        } else {
            this._ctx.moveTo(this._prevPoint.x, this._prevPoint.y);
        }

        this._ctx.lineTo(x, y);

        this._prevPoint = { x, y };
    }

    set drawMode(mode: CanvasDrawMode) {
        this._drawMode = mode;
        this._ctx.globalCompositeOperation =
            mode === "draw" ? "source-over" : "destination-out";
    }

    set lineColor(color: string) {
        this._ctx.strokeStyle = color;
    }

    set lineWidth(width: number) {
        this._ctx.lineWidth = width;
    }

    set fillStyle(color: string) {
        this._ctx.fillStyle = color;
    }
}

function onCanvasPointerDown() {
    if (
        !STATE.canvas ||
        STATE.binaryProtocolVersion === null ||
        STATE.socket.connectionState !== "connected"
    ) {
        return;
    }

    STATE.socket.ws.send(
        new Uint8Array([
            STATE.binaryProtocolVersion,
            ClientToServerEvents.PointerDown,
        ]),
    );
}

function onWindowPointerMove(e: MouseEvent) {
    if (
        !STATE.canvas ||
        STATE.binaryProtocolVersion === null ||
        !STATE.canvas.isDrawing ||
        STATE.socket.connectionState !== "connected"
    ) {
        return;
    }

    const clientX = e.clientX;
    const clientY = e.clientY;
    const canvasRect = STATE.canvas._ctx.canvas.getBoundingClientRect();
}

function onCanvasPointerLeave() {
    if (
        !STATE.canvas ||
        STATE.binaryProtocolVersion === null ||
        STATE.socket.connectionState !== "connected"
    ) {
        return;
    }

    STATE.socket.ws.send(
        new Uint8Array([
            STATE.binaryProtocolVersion,
            ClientToServerEvents.PointerLeave,
        ]),
    );
}

function onWindowPointerUp() {
    if (
        !STATE.canvas ||
        STATE.binaryProtocolVersion === null ||
        STATE.socket.connectionState !== "connected"
    ) {
        return;
    }

    STATE.socket.ws.send(
        new Uint8Array([
            STATE.binaryProtocolVersion,
            ClientToServerEvents.PointerUp,
        ]),
    );
}
