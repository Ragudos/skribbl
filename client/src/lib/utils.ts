export function processError(err: unknown) {
    if (err instanceof Error) {
        return err.message;
    }

    return typeof err === "string" ? err : "An unknown error occurred";
}
