import {
    Canvas,
    canvasPointerDownListener,
    canvasPointerLeaveListener,
    canvasPointerMoveListener,
    windowPointerUpListenerForCanvas,
} from "./canvas";
import {
    addUserToListOfPlayersElement,
    getDrawingCanvas,
    getListOfChatsContainer,
    getListOfPlayersElement,
    getTimeLeftEl,
    getWordList,
    getWordToDrawEl,
    onWordListBtnClick,
    removeUserFromListOfPlayersElement,
    setClientAsHostIfTrue,
    setUserToDraw,
    showRoom,
    togglePickingAWordModal,
} from "./dom";
import { toast } from "./lib/toast";
import { HTMLElementListener } from "./listener";
import { STATE } from "./state";
import {
    assert,
    parseObjAsRoomObj,
    parseObjAsUserObj,
    parsePartOfBinaryData,
} from "./utils";
import { startGameBtnListener } from "./waiting-room";

export function handleError(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    toast.error(parsePartOfBinaryData(data, "string"));
}

export function handleConnectError(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    toast.error(parsePartOfBinaryData(data, "string"));
    STATE.socket.ws.close();
}

export function handleUserJoined(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    const userString = parsePartOfBinaryData(data, "string");
    const user = JSON.parse(userString);

    if (user.ranking === undefined) {
        user.ranking = STATE.usersInRoom.length + 1;
    }

    if (!parseObjAsUserObj(user)) {
        throw new Error("Received invalid payload");
    }

    STATE.usersInRoom.push(user);
    addUserToListOfPlayersElement(user);
    getListOfPlayersElement().setAttribute(
        "data-player-count",
        STATE.usersInRoom.length.toString(),
    );
    toast(`${user.displayName} has joined the room.`);
}

export function handleUserLeft(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    const userId = parsePartOfBinaryData(data, "string");
    const userIdx = STATE.usersInRoom.findIndex((user) => {
        return user.id === userId;
    });
    const user = STATE.usersInRoom[userIdx];

    if (userIdx === -1) {
        console.error(
            "Received `UserLeft` event with a non-existent user id as its payload",
        );

        return;
    }

    for (let i = 0; i < STATE.usersInRoom.length; ++i) {
        if (
            user.id === STATE.usersInRoom[i].id ||
            user.ranking > STATE.usersInRoom[i].ranking
        ) {
            continue;
        }

        const rankingEl = document.querySelector(
            `#user-${STATE.usersInRoom[i].id} .ranking`,
        )!;

        assert(
            rankingEl !== null,
            `Element with class \`ranking\` whose parent has id \`user-${STATE.usersInRoom[i].id}\` does not exist.`,
        );

        rankingEl.setAttribute(
            "data-ranking",
            (--STATE.usersInRoom[i].ranking).toString(),
        );
    }

    STATE.usersInRoom.splice(userIdx, 1);
    removeUserFromListOfPlayersElement(userId);
    getListOfPlayersElement().setAttribute(
        "data-player-count",
        STATE.usersInRoom.length.toString(),
    );
    toast(`${user.displayName} has left the room.`);
}

export function handleStartGame(_data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room) {
        throw new Error(
            "Received event `startGame` despite room state being empty.",
        );
    }

    // Default state
    STATE.room.state = {
        playing: {
            playingState: {
                pickingAWord: {
                    wordsToPick: ["", "", ""],
                },
            },
            currentUserId: "",
            currentRound: 0,
        },
    };

    startGameBtnListener.disconnect();
    showRoom("playing-room");
}

export function handlePickAWord(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room || !STATE.user || STATE.usersInRoom.length === 0) {
        throw new Error("Received event `newTurn` despite state being empty.");
    }

    if (STATE.room.state === "waiting" || STATE.room.state === "finished") {
        console.error(
            "Received event `pickAWord` despite room not in playing state.",
        );

        return;
    }

    if (STATE.room.state.playing.currentUserId !== STATE.user.id) {
        console.error(
            "Received event `pickAWord` despite client not being the one to draw.",
        );

        return;
    }

    STATE.canvas?.destroy();
    canvasPointerDownListener.disconnect();
    canvasPointerLeaveListener.disconnect();
    canvasPointerMoveListener.disconnect();
    windowPointerUpListenerForCanvas.disconnect();

    const stringifiedWords = parsePartOfBinaryData(data, "string");
    const words = JSON.parse(stringifiedWords);

    if (!parseAsTupleOfThreeStrings(words)) {
        throw new Error("Received invalid payload");
    }

    STATE.room.state.playing.playingState = {
        pickingAWord: {
            wordsToPick: words,
        },
    };

    getWordList().innerHTML = "";

    const buttonListeners: HTMLElementListener<"click">[] = [];

    for (let i = 0; i < words.length; ++i) {
        const li = document.createElement("li");

        const buttonId = `word-${words[i]}`;
        const button = document.createElement("button");

        button.value = words[i];
        button.id = buttonId;
        button.textContent = words[i];

        li.appendChild(button);
        getWordList().appendChild(li);

        const listener = new HTMLElementListener(
            buttonId,
            "click",
            onWordListBtnClick,
        );

        listener.listen();
        buttonListeners.push(listener);
    }

    STATE.wordListBtnListeners =
        buttonListeners as typeof STATE.wordListBtnListeners;
    togglePickingAWordModal(true);
}

function parseAsTupleOfThreeStrings(
    strings: unknown,
): strings is [string, string, string] {
    if (!Array.isArray(strings) || strings.length !== 3) {
        return false;
    }

    return !strings.some((string) => {
        return typeof string !== "string";
    });
}

export function handleEndGame(_data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room) {
        throw new Error(
            "Received event `endGame` despite room state being empty.",
        );
    }

    if (STATE.room.state === "waiting" || STATE.room.state === "finished") {
        throw new Error("");
    }

    canvasPointerDownListener.disconnect();
    canvasPointerMoveListener.disconnect();
    windowPointerUpListenerForCanvas.disconnect();
    canvasPointerLeaveListener.disconnect();
    STATE.canvas?.destroy();
    STATE.canvas = null;

    getListOfPlayersElement()
        .querySelectorAll("li")
        .forEach((li) => {
            li.removeAttribute("data-guessed");
        });

    getWordToDrawEl().textContent = "";
    getTimeLeftEl().textContent = "";

    STATE.room.state = "finished";

    showRoom("finished-room");
}

export function handleResetRoom(_data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room) {
        throw new Error(
            "Received event `resetRoom` despite room state being empty.",
        );
    }

    if (STATE.room.state !== "finished" && STATE.room.state !== "waiting") {
        canvasPointerDownListener.disconnect();
        canvasPointerMoveListener.disconnect();
        windowPointerUpListenerForCanvas.disconnect();
        canvasPointerLeaveListener.disconnect();
    }

    STATE.room.state = "waiting";

    STATE.canvas?.destroy();
    STATE.canvas = null;

    if (STATE.wordListBtnListeners) {
        for (let i = 0; i < STATE.wordListBtnListeners.length; ++i) {
            STATE.wordListBtnListeners[i].disconnect();
        }

        STATE.wordListBtnListeners = null;

        getWordList().innerHTML = "";
        togglePickingAWordModal(false);
    }

    showRoom("waiting-room");
    // If we reset the room, the host will be the only player left anyway
    startGameBtnListener.listen();
}

export function handleNewTurn(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room || !STATE.user || STATE.usersInRoom.length === 0) {
        throw new Error("Received event `newTurn` despite state being empty.");
    }

    if (STATE.room.state === "waiting" || STATE.room.state === "finished") {
        console.error(
            "Received event `newTurn` despite room not in playing state.",
        );
        return;
    }

    STATE.canvas?.destroy();
    canvasPointerDownListener.disconnect();
    canvasPointerLeaveListener.disconnect();
    canvasPointerMoveListener.disconnect();
    windowPointerUpListenerForCanvas.disconnect();

    getListOfPlayersElement()
        .querySelectorAll("li")
        .forEach((li) => {
            li.removeAttribute("data-guessed");
        });

    getWordToDrawEl().textContent = "";

    const userId = parsePartOfBinaryData(data, "string");

    STATE.room.state.playing.currentUserId = userId;

    setUserToDraw();
}

export function handleNewWord(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (
        !STATE.room ||
        !STATE.user ||
        STATE.room.state === "waiting" ||
        STATE.room.state === "finished"
    ) {
        throw new Error(
            "Received event `newWord` despite state being empty or room not in playing state.",
        );
    }

    const word = parsePartOfBinaryData(data, "string");

    getWordToDrawEl().textContent = word;
    togglePickingAWordModal(false);
    STATE.canvas = new Canvas(getDrawingCanvas().getContext("2d")!);

    if (STATE.user.id === STATE.room.state.playing.currentUserId) {
        canvasPointerDownListener.listen();
        canvasPointerMoveListener.listen();
        windowPointerUpListenerForCanvas.listen();
        canvasPointerLeaveListener.listen();
    }
}

export function handleNewRound(data: Array<number>) {}

export function handleNewHost(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room) {
        throw new Error(
            "Received event `newHost` despite room state being empty.",
        );
    }

    const hostId = parsePartOfBinaryData(data, "string");

    STATE.room.hostId = hostId;

    if (setClientAsHostIfTrue()) {
        toast("You are now the host of this room.");
    }
}

export function handlePointerDown(_data: Array<number>) {
    if (!STATE.canvas) {
        return;
    }

    STATE.canvas.isDrawing = true;
}

export function handlePointerMove(data: Array<number>) {
    if (!STATE.canvas) {
        return;
    }

    const x = parsePartOfBinaryData(data, "float64");
    const y = parsePartOfBinaryData(data, "float64");

    STATE.canvas.drawLine(x, y);
}

export function handlePointerUp(_data: Array<number>) {
    if (!STATE.canvas) {
        return;
    }

    STATE.canvas.isDrawing = false;
    STATE.canvas.resetPrevPoint();
}

export function handlePointerLeave(_data: Array<number>) {
    if (!STATE.canvas) {
        return;
    }

    STATE.canvas.resetPrevPoint();
}

export function handleChangeColor(data: Array<number>) {}

export function handleSendGameState(data: Array<number>) {
    if (STATE.room || STATE.user || STATE.usersInRoom.length !== 0) {
        throw new Error(
            "Received one whole game state despite state not being empty.",
        );
    }

    const roomString = parsePartOfBinaryData(data, "string");
    const userString = parsePartOfBinaryData(data, "string");
    const usersInRoomString = parsePartOfBinaryData(data, "string");

    const room = JSON.parse(roomString);
    const user = JSON.parse(userString);
    const usersInRoom = JSON.parse(usersInRoomString).map(
        (user: any, idx: number) => ({ ...user, ranking: idx + 1 }),
    );

    if (
        !parseObjAsRoomObj(room) ||
        !parseObjAsUserObj(user) ||
        !Array.isArray(usersInRoom) ||
        !usersInRoom.some((user) => parseObjAsUserObj(user))
    ) {
        throw new Error("Received invalid payload from server");
    }

    STATE.room = room;
    STATE.user = user;
    STATE.usersInRoom = usersInRoom;
}

export function handleSendMessage(data: Array<number>) {
    if (!STATE.room || !STATE.user || STATE.usersInRoom.length === 0) {
        throw new Error();
    }

    const senderId = parsePartOfBinaryData(data, "string");
    const message = parsePartOfBinaryData(data, "string");
    const sender = STATE.usersInRoom.find((user) => {
        return user.id == senderId;
    });

    if (!sender) {
        throw new Error();
    }

    const li = document.createElement("li");
    const name = document.createElement("div");

    name.textContent = sender.displayName + ": ";
    name.classList.add("name");

    li.appendChild(name);

    const messageEl = document.createElement("p");

    messageEl.textContent = message;
    li.appendChild(messageEl);

    getListOfChatsContainer().appendChild(li);

    li.scrollIntoView({ behavior: "smooth", block: "end" });
}

export function handleTick(data: Array<number>) {
    if (
        !STATE.room ||
        STATE.room.state === "waiting" ||
        STATE.room.state === "finished"
    ) {
        throw new Error();
    }

    const timeLeft = parsePartOfBinaryData(data, "int8");

    getTimeLeftEl().textContent = timeLeft + "s";
}

export function handleAddScore(data: Array<number>) {
    assert(
        STATE.socket.connectionState === "connected",
        "Received `AddScore` event but `STATE.connectionState` is not `connected`",
    );

    const userId = parsePartOfBinaryData(data, "string");
    const score = parsePartOfBinaryData(data, "int16");
    const user = STATE.usersInRoom.find((user) => user.id === userId)!;

    assert(
        user !== undefined,
        "User with id " + userId + " does not exist in state.",
    );

    user.score = score;
}

export function handleUserGuessed(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        throw new Error();
    }

    if (
        !STATE.room ||
        STATE.room.state === "waiting" ||
        STATE.room.state === "finished" ||
        !STATE.user ||
        STATE.usersInRoom.length === 0
    ) {
        throw new Error();
    }

    const userId = parsePartOfBinaryData(data, "string");
    const userEl = document.getElementById(`user-${userId}`)!;

    userEl.setAttribute("data-guessed", "true");
}

export function handleSystemMessage(data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        throw new Error();
    }

    if (
        !STATE.room ||
        STATE.room.state === "waiting" ||
        STATE.room.state === "finished" ||
        !STATE.user ||
        STATE.usersInRoom.length === 0
    ) {
        throw new Error();
    }

    const message = parsePartOfBinaryData(data, "string");

    const li = document.createElement("li");
    const messageEl = document.createElement("div");

    li.setAttribute("data-message", "system");
    messageEl.textContent = message;

    li.appendChild(messageEl);

    getListOfChatsContainer().appendChild(li);

    li.scrollIntoView({ behavior: "smooth", block: "end" });
}

export function handleRevealWord(data: Array<number>) {
    const word = parsePartOfBinaryData(data, "string");

    getWordToDrawEl().textContent = word;
}
