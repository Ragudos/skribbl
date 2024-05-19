import { PlayingState, Room, User } from "./types";

type DataTypes = {
    string: string;
    float32: number;
    float64: number;
    int8: number;
    int16: number;
    int32: number;
    uint8: number;
    uint16: number;
    uint32: number;
    bigint64: bigint;
    biguint64: bigint;
};

export function parsePartOfBinaryData<K extends keyof DataTypes>(
    data: Array<number>,
    dataType: K,
): DataTypes[K] {
    const lengthSpace = data.splice(0, 1)[0];
    const length = data
        .splice(0, lengthSpace)
        .reduce((acc, curr) => acc + curr);
    const bytes = new Uint8Array(data.splice(0, length));

    switch (dataType) {
        case "string": {
            return new TextDecoder().decode(bytes) as DataTypes[K];
        }
        case "float32": {
            return new DataView(bytes.buffer).getFloat32(0) as DataTypes[K];
        }
        case "float64": {
            return new DataView(bytes.buffer).getFloat64(0) as DataTypes[K];
        }
        case "int8": {
            return new DataView(bytes.buffer).getInt8(0) as DataTypes[K];
        }
        case "int16": {
            return new DataView(bytes.buffer).getInt16(0) as DataTypes[K];
        }
        case "int32": {
            return new DataView(bytes.buffer).getInt32(0) as DataTypes[K];
        }
        case "uint8": {
            return new DataView(bytes.buffer).getUint8(0) as DataTypes[K];
        }
        case "uint16": {
            return new DataView(bytes.buffer).getUint16(0) as DataTypes[K];
        }
        case "uint32": {
            return new DataView(bytes.buffer).getUint32(0) as DataTypes[K];
        }
        case "bigint64": {
            return new DataView(bytes.buffer).getBigInt64(0) as DataTypes[K];
        }
        case "biguint64": {
            return new DataView(bytes.buffer).getBigUint64(0) as DataTypes[K];
        }
        default: {
            throw new Error("Invalid data type");
        }
    }
}

export function getApproximateCursorPositionInCanvas(
    clientPos: number,
    rectPos: number,
    clientDim: number,
    rectDim: number,
) {
    return (clientPos - rectPos) * (clientDim / rectDim);
}

export function processError(err: unknown) {
    if (err instanceof Error) {
        return err.message;
    }

    return typeof err === "string" ? err : "An unknown error occurred";
}

export function parseObjAsRoomObj(obj: unknown): obj is Room {
    if (typeof obj !== "object" || !obj) {
        return false;
    }

    return (
        "id" in obj &&
        typeof obj.id === "string" &&
        "hostId" in obj &&
        typeof obj.hostId === "string" &&
        "visibility" in obj &&
        (obj.visibility === "public" || obj.visibility === "private") &&
        "state" in obj &&
        (obj.state === "waiting" ||
            obj.state === "finished" ||
            parseObjAsPlayingStateObj(obj.state)) &&
        "maxUsers" in obj &&
        typeof obj.maxUsers === "number" &&
        "maxRounds" in obj &&
        typeof obj.maxRounds === "number"
    );
}

export function parseObjAsUserObj(obj: unknown): obj is User {
    if (typeof obj !== "object" || !obj) {
        return false;
    }

    return (
        "id" in obj &&
        typeof obj.id === "string" &&
        "displayName" in obj &&
        typeof obj.displayName === "string"
    );
}

function parseObjAsPlayingStateObj(obj: unknown): obj is PlayingState {
    if (typeof obj !== "object" || !obj) {
        return false;
    }

    if ("drawing" in obj) {
        if (typeof obj.drawing !== "object" || !obj.drawing) {
            return false;
        }

        return (
            "currentWord" in obj.drawing &&
            typeof obj.drawing.currentWord === "string"
        );
    }

    if ("pickingAWord" in obj) {
        if (typeof obj.pickingAWord !== "object" || !obj.pickingAWord) {
            return false;
        }

        return (
            "wordsToPick" in obj.pickingAWord &&
            Array.isArray(obj.pickingAWord.wordsToPick) &&
            obj.pickingAWord.wordsToPick.length === 3
        );
    }

    return false;
}
