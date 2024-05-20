import { Room, PlayingState, User } from "./types";

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

export function turnNumberToArrayOfU8Int(num: number): Array<number> {
	let amountLeft = num;
	const arr: number[] = [];

	while (amountLeft !== 0) {
		if (amountLeft < 255) {
			const diff = amountLeft - amountLeft;

			arr.push(amountLeft);
			amountLeft = diff;
		} else {
			amountLeft -= 255;
			arr.push(255);
		}
	}

	return arr;
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

    if (
        !("playing" in obj) ||
        typeof obj.playing !== "object" ||
        !obj.playing
    ) {
        return false;
    }

    if (
        !("playingState" in obj.playing) ||
        typeof obj.playing.playingState !== "object" ||
        !obj.playing.playingState
    ) {
        return false;
    }

    if (
        !("currentUserId" in obj.playing) ||
        typeof obj.playing.currentUserId !== "string"
    ) {
        return false;
    }

    if (
        !("currentRound" in obj.playing) ||
        typeof obj.playing.currentRound !== "number"
    ) {
        return false;
    }

    if ("drawing" in obj.playing.playingState) {
        if (
            typeof obj.playing.playingState.drawing !== "object" ||
            !obj.playing.playingState.drawing
        ) {
            return false;
        }

        return (
            "currentWord" in obj.playing.playingState.drawing &&
            typeof obj.playing.playingState.drawing.currentWord === "string"
        );
    }

    if ("pickingAWord" in obj.playing.playingState) {
        if (
            typeof obj.playing.playingState.pickingAWord !== "object" ||
            !obj.playing.playingState.pickingAWord
        ) {
            return false;
        }

        return (
            "wordsToPick" in obj.playing.playingState.pickingAWord &&
            Array.isArray(obj.playing.playingState.pickingAWord.wordsToPick) &&
            obj.playing.playingState.pickingAWord.wordsToPick.length === 3
        );
    }

    return false;
}
