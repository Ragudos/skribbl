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

/**
 * Parses a part of a buffer of data into a JavaScript data type.
 *
 * For example:
 *
 * [1, 2, 20, 50]
 *
 * The first number is the amount of bytes the length of the data type is.
 * If the length is more than one u8 byte (255 characters), it will take up
 * more than one byte.
 *
 * The second number is the length of the data. This can take up
 * more than one byte.
 *
 * The rest of the bytes is the data itelf.
 *
 * This splices the `Array<number>` until
 * the end of the data type's bytes, leaving the original
 * buffer with only the remaining data if there's any left.
 */
export function parseData<K extends keyof DataTypes>(
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
