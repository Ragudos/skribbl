import type { Canvas } from "./canvas";
import { HTMLElementListener, WebSocketListener } from "./listener";
import { Ticker } from "./timer";

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
              playingState: PlayingState;
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
          listeners: {
              onmessage: WebSocketListener<"message">;
              onclose: WebSocketListener<"close">;
              onerror: WebSocketListener<"error">;
          };
      };

export type GameState = {
    socket: Socket;
    user: User | null;
    room: Room | null;
    usersInRoom: User[];
    binaryProtocolVersion: number | null;
    canvas: Canvas | null;
    wordListBtnListeners:
        | null
        | [
              HTMLElementListener<"click">,
              HTMLElementListener<"click">,
              HTMLElementListener<"click">,
          ];
    ticker: Ticker | null;
};

export enum ClientToServerEvents {
    StartGame = 0,
    PickAWord = 1,
    PointerDown = 2,
    PointerMove = 3,
    PointerUp = 4,
    PointerLeave = 5,
    ChangeColor = 6,
    Message = 7,
}

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
    NewRound = 10,
    NewHost = 11,
    PointerDown = 12,
    PointerMove = 13,
    PointerUp = 14,
    PointerLeave = 15,
    ChangeColor = 16,
    SendGameState = 17,
    SendMessage = 18,
    AddScore = 19,
    Tick = 20,
    UserGuessed = 21,
    SystemMessage = 22,
    RevealWord = 23,
}
