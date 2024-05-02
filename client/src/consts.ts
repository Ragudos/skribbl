export const MAX_DISPLAY_NAME_LENGTH = 20;
export const MIN_DISPLAY_NAME_LENGTH = 3;
export const wsProtocol = import.meta.env.DEV ? "ws" : "wss";
export const wsHost = import.meta.env.DEV
    ? "localhost:3000"
    : "skribbl.aaronragudos.com";
