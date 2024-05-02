export type RoomState =
    | "waiting"
    | {
          playing: {
              userToDraw: string;
          };
      }
    | "finished";
export type Visibility = "public" | "private";

export type Room = {
    id: string;
    hostId: string;
    visibility: Visibility;
    state: RoomState;
};

export type User = {
    id: string;
    displayName: string;
};

export type HandshakePayload = {
    user: User;
    room: Room;
};

export type Socket =
    | {
          connectionState: "connecting" | "disconnected";
          ws: null;
      }
    | {
          connectionState: "connected";
          ws: WebSocket;
      };

export type GameState = {
    socket: Socket;
    user: User | null;
    room: Room | null;
};
