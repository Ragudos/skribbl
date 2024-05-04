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
    maxUsers: number;
    maxRounds: number;
};

export type User = {
    id: string;
    displayName: string;
};

export type HandshakePayload = {
    user: User;
    room: Room;
    usersInRoom: User[];
    binaryProtocolVersion: number;
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
    usersInRoom: User[];
    binaryProtocolVersion: number | null;
};

export enum WebSocketEvents {
    Error = 0,
    UserJoined = 1,
    UserLeft = 2,
    StartGame = 3,
    EndGame = 4,
    NewRound = 5,
    NewUserToDraw = 6,
    PointerDown = 7,
    PointerMove = 8,
    PointerUp = 9,
    ChangeColor = 10,
    Tick = 11,
    ResetRoom = 12,
    NewHost = 13,
}
