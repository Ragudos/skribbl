type TickerCb = (timeLeft: number) => void;

export class Ticker {
    private _cb: TickerCb;
    private _totalTime: number;
    private _timeout: number | null;

    constructor(cb: TickerCb, totalTime: number) {
        this._cb = cb;
        this._totalTime = totalTime;

        this._timeout = null;
    }

    start(): void {
        if (this._timeout !== null) {
            return;
        }

        this._timeout = setTimeout(this._loop.bind(this), 1_000);
    }

    stop(): void {
        if (this._timeout === null) {
            return;
        }

        clearTimeout(this._timeout);
        this._timeout = null;
    }

    private _loop(): void {
        this._cb(--this._totalTime);

        if (this._timeout === null) {
            return;
        }

        if (this._totalTime === 0) {
            this._timeout = null;

            return;
        }

        this._timeout = setTimeout(this._loop.bind(this), 1_000);
    }
}
