import { chatFormListener } from "./chats";
import { toast } from "./lib/toast";
import { STATE } from "./state";
import { ClientToServerEvents, User } from "./types";
import { assert, turnNumberToArrayOfU8Int } from "./utils";

export function showRoom(
    roomId: "lobby" | "waiting-room" | "playing-room" | "finished-room",
) {
    const activeRoom = document.querySelector(
        ".rooms:not([hidden])",
    ) as HTMLElement;

    activeRoom.setAttribute("hidden", "");
    document.getElementById(roomId)!.removeAttribute("hidden");
}

export function initializeWaitingRoom() {
    if (!STATE.room || !STATE.user || STATE.usersInRoom.length === 0) {
        throw new Error(
            "Initializing waiting room, but internal state for `room`, `user`, and `usersInRoom` does not exist.",
        );
    }

    getRoomLinkInputElement().value = `${window.location.origin}?roomId=${STATE.room.id}`;
    populateListOfPlayers();
    setClientAsHostIfTrue();

    getRoomLinkElement().removeAttribute("hidden");
    getListOfPlayersElement().removeAttribute("hidden");
    chatFormListener.listen();
    showRoom("waiting-room");
}

export function setUserToDraw() {
    if (!STATE.user) {
        console.error("Calling `setUserToDraw` despite user not existing.");
        return;
    }

    if (
        !STATE.room ||
        STATE.room.state === "waiting" ||
        STATE.room.state === "finished"
    ) {
        console.error(
            "Calling `setUserToDraw` despite room not existing or not in playing state.",
        );
        return;
    }

    if (STATE.room.state.playing.currentUserId === STATE.user.id) {
        document.body.setAttribute("data-user-to-draw", "true");
    } else {
        if (document.body.hasAttribute("data-user-to-draw")) {
            document.body.removeAttribute("data-user-to-draw");
        }

        getUserToDrawUsername().textContent =
            STATE.usersInRoom.find((user) => {
                if (
                    !STATE.room ||
                    STATE.room.state === "waiting" ||
                    STATE.room.state === "finished"
                ) {
                    return false;
                }

                return STATE.room.state.playing.currentUserId === user.id;
            })?.displayName ?? "";
        togglePickingAWordModal(true);
    }
}

export function onWordListBtnClick(e: Event) {
    if (STATE.socket.connectionState !== "connected") {
        throw new Error(
            "Received event `click` from a button in `word-list` despite socket not being in `connected` connectionState",
        );
    }

    if (STATE.binaryProtocolVersion === null) {
        return;
    }

    if (!STATE.wordListBtnListeners) {
        throw new Error(
            "Received event `click` from a button in `word-list` despite its listener not in state",
        );
    }

    const value = (e.currentTarget as HTMLButtonElement).value;

    for (let i = 0; i < STATE.wordListBtnListeners.length; ++i) {
        STATE.wordListBtnListeners[i].disconnect();
    }

    STATE.wordListBtnListeners = null;

    const wordBinary = new TextEncoder().encode(value);
    const lengthInU8 = turnNumberToArrayOfU8Int(wordBinary.length);

    STATE.socket.ws.send(
        new Uint8Array([
            STATE.binaryProtocolVersion,
            ClientToServerEvents.PickAWord,
            lengthInU8.length,
            ...lengthInU8,
            ...wordBinary,
        ]),
    );
}

export function populateListOfPlayers() {
    if (!STATE.room || !STATE.user || STATE.usersInRoom.length === 0) {
        throw new Error(
            "Populating list of players, but internal state for `room`, `user`, and `usersInRoom` does not exist.",
        );
    }

    for (let i = 0; i < STATE.usersInRoom.length; ++i) {
        addUserToListOfPlayersElement(STATE.usersInRoom[i]);
    }

    getListOfPlayersElement().setAttribute(
        "data-player-count",
        STATE.usersInRoom.length.toString(),
    );
}

export function clearListOfPlayers() {
    getListOfPlayersElement().innerHTML = "";
    getListOfPlayersElement().removeAttribute("data-player-count");
}

export function resetBodyAttributes() {
    document.body.removeAttribute("data-host");
}

export function setClientAsHostIfTrue() {
    if (!STATE.room || !STATE.user || STATE.usersInRoom.length === 0) {
        throw new Error(
            "Trying to set client as host of a room, but internal state for `room`, `user`, and `usersInRoom` does not exist.",
        );
    }

    if (STATE.user.id !== STATE.room.hostId) {
        if (document.body.getAttribute("data-host") === "true") {
            document.body.removeAttribute("data-host");

            const userEl = document.getElementById(`user-${STATE.user.id}`);

            if (userEl) {
                userEl.querySelector("[data-host-badge]")?.remove();
            }
        }

        return false;
    }

    document.body.setAttribute("data-host", "true");

    const userEl = document.getElementById(`user-${STATE.user.id}`);

    if (userEl && !userEl.querySelector("[data-host-badge]")) {
        const hostBadge = document.createElement("span");

        hostBadge.setAttribute("data-host-badge", "");
        hostBadge.classList.add("badge");
        hostBadge.textContent = "🜲";

        userEl.querySelector(".user-metadata")!.appendChild(hostBadge);
    }

    return true;
}

export function addUserToListOfPlayersElement(user: User): void {
    const li = document.createElement("li");
    const metadataContainer = document.createElement("div");
    const displayNameEl = document.createElement("div");
    const mainInfoContainer = document.createElement("div");

    li.id = `user-${user.id}`;
    metadataContainer.classList.add("user-metadata");
    displayNameEl.textContent = user.displayName;
    metadataContainer.appendChild(displayNameEl);

    if (user.id === STATE.room!.hostId) {
        const hostBadge = document.createElement("span");

        hostBadge.setAttribute("data-host-badge", "");
        hostBadge.classList.add("badge");
        hostBadge.textContent = "🜲";
        metadataContainer.appendChild(hostBadge);
    }

    if (STATE.user?.id === user.id) {
        const youBadge = document.createElement("sub");

        youBadge.classList.add("badge");
        youBadge.textContent = "(you)";
        metadataContainer.appendChild(youBadge);
    }

    const ranking = document.createElement("div");
    const score = document.createElement("div");

    assert(
        user.ranking !== undefined,
        "User object should have property `ranking`",
    );

    ranking.setAttribute("data-ranking", user.ranking!.toString());
    ranking.classList.add("ranking");
    score.textContent = "0 points";
    score.classList.add("score");
    mainInfoContainer.appendChild(metadataContainer);
    mainInfoContainer.appendChild(score);
    li.appendChild(ranking);
    li.appendChild(mainInfoContainer);
    getListOfPlayersElement().appendChild(li);
}

export function removeUserFromListOfPlayersElement(userId: string): void {
    document.getElementById(`user-${userId}`)?.remove();
}

export function togglePickingAWordModal(show: boolean) {
    if (show) {
        getPickingAWordModal().setAttribute("data-visible", "true");
    } else {
        getPickingAWordModal().removeAttribute("data-visible");
    }
}

export function getListOfPlayersElement(): HTMLUListElement {
    const listOfPlayersEl = document.getElementById("list-of-players");

    if (!listOfPlayersEl || !(listOfPlayersEl instanceof HTMLUListElement)) {
        throw new Error(
            "Element with id `list-of-players` cannot be found or it's not an instance of HTMLUListElement",
        );
    }

    return listOfPlayersEl;
}

export function getRoomLinkElement(): HTMLElement {
    const roomLink = document.getElementById("room-link");

    if (!roomLink || !(roomLink instanceof HTMLElement)) {
        throw new Error(
            "Element with id `room-link` cannot be found or it's not an instance of HTMLElement",
        );
    }

    return roomLink;
}

export function getRoomLinkInputElement(): HTMLInputElement {
    const roomLinkInput = document.getElementById("room-link-input");

    if (!roomLinkInput || !(roomLinkInput instanceof HTMLInputElement)) {
        throw new Error(
            "Element with id `room-link-input` cannot be found or it's not an instance of a HTMLInputElement",
        );
    }

    return roomLinkInput;
}

export function getWordList(): HTMLUListElement {
    const wordList = document.getElementById("word-list");

    if (!wordList || !(wordList instanceof HTMLUListElement)) {
        throw new Error(
            "Element with id `word-list` cannot be found or it's not an instance of HTMLUListElement",
        );
    }

    return wordList;
}

export function getUserToDrawUsername(): HTMLElement {
    const userToDrawUsername = document.getElementById("user-to-draw-username");

    if (!userToDrawUsername || !(userToDrawUsername instanceof HTMLElement)) {
        throw new Error(
            "Element with id `user-to-draw-username` cannot be found or it's not an instance of a HTMLElement",
        );
    }

    return userToDrawUsername;
}

export function getPickingAWordModal(): HTMLElement {
    const pickingAWordModal = document.getElementById("picking-a-word-modal");

    if (!pickingAWordModal || !(pickingAWordModal instanceof HTMLElement)) {
        throw new Error(
            "Element with id `picking-a-word-modal` cannot be found or it's not an instance of a HTMLElement",
        );
    }

    return pickingAWordModal;
}

export function getWordToDrawEl(): HTMLElement {
    const wordToDrawEl = document.getElementById("word-to-draw");

    if (!wordToDrawEl || !(wordToDrawEl instanceof HTMLElement)) {
        throw new Error(
            "Element with id `word-to-draw` cannot be found or it's not an instance of a HTMLElement",
        );
    }

    return wordToDrawEl;
}

export function getTimeLeftEl(): HTMLElement {
    const timeLeftEl = document.getElementById("time-left");

    if (!timeLeftEl || !(timeLeftEl instanceof HTMLElement)) {
        throw new Error(
            "Element with id `time-left` cannot be found or it's not an instance of a HTMLElement",
        );
    }

    return timeLeftEl;
}

export function getDrawingCanvas(): HTMLCanvasElement {
    const canvas = document.getElementById("drawing-canvas");

    if (!canvas || !(canvas instanceof HTMLCanvasElement)) {
        throw new Error(
            "Element with id `drawing-canvas` cannot be found or it's not an instance of HTMLCanvasElement",
        );
    }

    return canvas;
}

export function getChatContainer(): HTMLElement {
    const chatContainer = document.getElementById("chat");

    if (!chatContainer || !(chatContainer instanceof HTMLElement)) {
        throw new Error(
            "Element with id `chat` cannot be found or it's not an instance of HTMLElement",
        );
    }

    return chatContainer;
}

export function getListOfChatsContainer(): HTMLUListElement {
    const listOfChatsContainer = document.getElementById("list-of-chats");

    if (
        !listOfChatsContainer ||
        !(listOfChatsContainer instanceof HTMLUListElement)
    ) {
        throw new Error(
            "Element with id `list-of-chats` cannot be found or it's not an instance of HTMLUListElement",
        );
    }

    return listOfChatsContainer;
}

export function getChatForm(): HTMLFormElement {
    const chatForm = document.getElementById("chat-form");

    if (!chatForm || !(chatForm instanceof HTMLFormElement)) {
        throw new Error(
            "Element with id `chat-form` cannot be found or it's not an instance of HTMLFormElement",
        );
    }

    return chatForm;
}

/**
 * @param copying id of the element to copy
 * @returns
 */
export function initializeCopyBtn(copying: string) {
    const button = document.querySelector(
        `[data-copy-btn=${copying}]`,
    ) as HTMLButtonElement;
    const inputToCopy = document.getElementById(copying) as
        | HTMLInputElement
        | HTMLTextAreaElement;
    let timeout: number | undefined;

    assert(
        button !== null && button instanceof HTMLButtonElement,
        `Copy button which targets element with id ${copying} does not exist nor an instance of HTMLButtonElement`,
    );
    assert(
        inputToCopy !== null &&
            (inputToCopy instanceof HTMLInputElement ||
                inputToCopy instanceof HTMLTextAreaElement),
        `Input to copy with id ${copying} is neither defined nor an instance of HTMLInputElement nor HTMLTextAreaElement`,
    );

    function onClick() {
        navigator.clipboard
            .writeText(inputToCopy.value)
            .then(() => {
                button.setAttribute("data-copied", "true");

                if (timeout) {
                    clearTimeout(timeout);
                }

                timeout = setTimeout(() => {
                    button.removeAttribute("data-copied");
                }, 2000);
            })
            .catch((err) => {
                toast.error(err);
            });
    }

    button.addEventListener("click", onClick);

    return () => {
        const button = document.querySelector(
            `[data-copy-btn=${copying}]`,
        ) as HTMLButtonElement;

        assert(
            button !== null && button instanceof HTMLButtonElement,
            `Copy button which targets element with id ${copying} does not exist nor an instance of HTMLButtonElement`,
        );

        button.removeEventListener("click", onClick);
    };
}
