/**
 * Parses a part of a buffer of data into text.
 *
 * For example:
 *
 * [1, 2, 20, 50]
 *
 * The first number is the amount of bytes the length of the text is.
 * If the length is more than one byte (255 characters), it will take up
 * more than one byte.
 *
 * The second number is the length of the data in text. This can take up
 * more than one byte.
 *
 * The rest of the data is the text.
 *
 * This splices the `Array<number>` until
 * the end of the text, leaving the original
 * buffer with only the remaining data if any.
 */
export function parseData(data: Array<number>) {
    const lengthSpace = data.splice(0, 1)[0];
    const length = data
        .splice(0, lengthSpace)
        .reduce((acc, curr) => acc + curr);
    return new TextDecoder().decode(new Uint8Array(data.splice(0, length)));
}
