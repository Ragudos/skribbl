import { GameState } from "./types";

export const STATE: GameState = {
    socket: {
        connectionState: "disconnected",
        ws: null,
    },

    user: null,
    room: null,

    usersInRoom: [],
    binaryProtocolVersion: null,
};
