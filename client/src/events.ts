import { Point } from "./canvas";
import {
    addUserToListOfPlayersElement,
    getListOfPlayersElement,
    getWordList,
    initializeWaitingRoom,
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
			"Received event `pickAWord` despite client not being the one to draw."
		);

		return;
	}

	const stringifiedWords = parsePartOfBinaryData(data, "string");
	const words = JSON.parse(stringifiedWords);

	if (!parseAsTupleOfThreeStrings(words)) {
		throw new Error("Received invalid payload");
	}

	STATE.room.state.playing.playingState = {
		pickingAWord: {
			wordsToPick: words
		}
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

		const listener = new HTMLElementListener(buttonId, "click", onWordListBtnClick);

		listener.listen();
		buttonListeners.push(listener);
	}

	STATE.wordListBtnListeners = buttonListeners as typeof STATE.wordListBtnListeners;
	togglePickingAWordModal(true);
}

function parseAsTupleOfThreeStrings(strings: unknown): strings is [string, string, string] {
	if (!Array.isArray(strings) || strings.length !== 3) {
		return false;
	}

	return !strings.some((string) => {
		return typeof string !== "string"
	});
}

export function handleEndGame(data: Array<number>) {}

export function handleResetRoom(_data: Array<number>) {
    if (STATE.socket.connectionState !== "connected") {
        return;
    }

    if (!STATE.room) {
        throw new Error(
            "Received event `resetRoom` despite room state being empty.",
        );
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

    const userId = parsePartOfBinaryData(data, "string");

    STATE.room.state.playing.currentUserId = userId;

    setUserToDraw();
}

export function handleNewWord(data: Array<number>) {
	const word = parsePartOfBinaryData(data, "string");

	console.log(word);
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

export function handlePointerDown(data: Array<number>) {}
export function handlePointerMove(data: Array<number>) {}
export function handlePointerUp(data: Array<number>) {}
export function handlePointerLeave(data: Array<number>) {}
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
    const usersInRoom = JSON.parse(usersInRoomString);

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

export function handleSendMessage(data: Array<number>) {}
