export type PlayingState =
    | {
          drawing: {
              currentWord: string;
          };
      }
    | {
          pickingAWord: {
              wordsToPick: [string, string, string];
          };
      };

export type RoomState =
    | "waiting"
    | {
          playing: {
              payingState: PlayingState;
              currentUserId: string;
              currentRound: number;
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

export enum ServerToClientEvents {
    Error = 0,
    ConnectError = 1,
    UserJoined = 2,
    UserLeft = 3,
    StartGame = 4,
    PickAWord = 5,
    EndGame = 6,
    ResetRoom = 7,
    NewTurn = 8,
    NewWord = 9,
    NewHost = 10,
    NewRound = 11,
    PointerDown = 12,
    PointerMove = 13,
    PointerUp = 14,
    ChangeColor = 15,
    SendUserInfo = 16,
    SendRoomInfo = 17,
    SendUsersInRoomInfo = 18,
    SendMessage = 19,
}
