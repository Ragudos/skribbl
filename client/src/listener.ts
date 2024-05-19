interface Listener {
    listen(): void;
    disconnect(): void;
}

export class WindowListener<K extends keyof WindowEventMap>
    implements Listener
{
    private _connectionTrigger: K;
    private _eventListener: (e: WindowEventMap[K]) => void;

    constructor(
        connectionTrigger: K,
        eventListener: (e: WindowEventMap[K]) => void,
    ) {
        this._connectionTrigger = connectionTrigger;
        this._eventListener = eventListener;
    }

    listen(): void {
        window.addEventListener(this._connectionTrigger, this._eventListener);
    }

    disconnect(): void {
        window.removeEventListener(
            this._connectionTrigger,
            this._eventListener,
        );
    }
}

export class HTMLElementListener<K extends keyof HTMLElementEventMap>
    implements Listener
{
    private _elId: string;
    private _connectionTrigger: K;
    private _eventListener: (e: HTMLElementEventMap[K]) => void;

    constructor(
        elId: string,
        connectionTrigger: K,
        eventListener: (e: HTMLElementEventMap[K]) => void,
    ) {
        this._elId = elId;
        this._connectionTrigger = connectionTrigger;
        this._eventListener = eventListener;
    }

    listen() {
        const element = document.getElementById(this._elId);

        if (!element) {
            throw new Error("Cannot find element with id: " + this._elId);
        }

        element.addEventListener(this._connectionTrigger, this._eventListener);
    }

    disconnect() {
        if (!this._eventListener) {
            return;
        }

        const element = document.getElementById(this._elId);

        if (!element) {
            throw new Error("Cannot find element with id: " + this._elId);
        }

        element.removeEventListener(
            this._connectionTrigger,
            this._eventListener,
        );
    }
}

export class WebSocketListener<K extends keyof WebSocketEventMap>
    implements Listener
{
    private _ws: WebSocket;
    private _connectionTrigger: K;
    private _eventListener: (e: WebSocketEventMap[K]) => void;

    constructor(
        ws: WebSocket,
        connectionTrigger: K,
        eventListener: (e: WebSocketEventMap[K]) => void,
    ) {
        this._ws = ws;
        this._connectionTrigger = connectionTrigger;
        this._eventListener = eventListener;
    }

    listen(): void {
        this._ws.addEventListener(this._connectionTrigger, this._eventListener);
    }

    disconnect(): void {
        this._ws.removeEventListener(
            this._connectionTrigger,
            this._eventListener,
        );
    }
}
