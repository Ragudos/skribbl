import { toast } from "./lib/toast";
import { STATE } from "./state";
import { User } from "./types";

export function showRoom(roomId: "lobby" | "waiting-room" | "playing-room") {
    const activeRoom = document.querySelector(
        ".rooms[data-current='true']",
    ) as HTMLElement;

    activeRoom.removeAttribute("data-current");
    document.getElementById(roomId)!.setAttribute("data-current", "true");
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
    showRoom("waiting-room");
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
        hostBadge.textContent = "(host)";

        userEl.appendChild(hostBadge);
    }

    return true;
}

export function addUserToListOfPlayersElement(user: User): void {
    const li = document.createElement("li");

    li.id = `user-${user.id}`;

    if (STATE.user?.id === user.id) {
        const youBadge = document.createElement("span");

        youBadge.classList.add("badge");
        youBadge.textContent = "(you)";

        li.appendChild(youBadge);
    }

    const displayNameEl = document.createElement("div");

    displayNameEl.textContent = user.displayName;

    li.appendChild(displayNameEl);

    if (user.id === STATE.room!.hostId) {
        const hostBadge = document.createElement("span");

        hostBadge.setAttribute("data-host-badge", "");
        hostBadge.classList.add("badge");
        hostBadge.textContent = "(host)";

        li.appendChild(hostBadge);
    }

    getListOfPlayersElement().appendChild(li);
}

export function removeUserFromListOfPlayersElement(userId: string): void {
    document.getElementById(`user-${userId}`)?.remove();
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
