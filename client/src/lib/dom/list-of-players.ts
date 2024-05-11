import { STATE } from "../../state";
import { User } from "../../types";

export function updateListOfPlayers() {
    const listOfPlayers = document.getElementById(
        "list-of-players",
    ) as HTMLElement;

    listOfPlayers.innerHTML = "";

    if (STATE.usersInRoom.length === 0) {
        listOfPlayers.setAttribute("hidden", "");

        return;
    }

    listOfPlayers.removeAttribute("hidden");

    for (let i = 0; i < STATE.usersInRoom.length; ++i) {
        const user = STATE.usersInRoom[i];

        addUserToDOM(user);
    }
}

export function playerLeft(userId: string) {
    if (STATE.usersInRoom.length === 0) {
        return;
    }

    const playerElement = document.getElementById(`player-${userId}`);

    if (!playerElement) {
        console.error(
            "Either received invalid user ID or user's info is not in the list of players in the DOM",
        );
        return;
    }

    playerElement.remove();

    if (STATE.usersInRoom.length === 0) {
        const listOfPlayers = document.getElementById(
            "list-of-players",
        ) as HTMLElement;

        listOfPlayers.setAttribute("hidden", "");
    }
}

export function addUserToDOM(user: User) {
    const listOfPlayers = document.getElementById(
        "list-of-players",
    ) as HTMLElement;

    const playerElement = document.createElement("li");
    playerElement.id = `player-${user.id}`;

    const playerNameContainer = document.createElement("div");
    playerNameContainer.textContent = user.displayName;

    playerElement.appendChild(playerNameContainer);

    if (STATE.user!.id === user.id) {
        const youBadge = document.createElement("span");
        youBadge.textContent = "You";
        youBadge.classList.add("badge");

        playerElement.appendChild(youBadge);
    }

    if (STATE.room!.hostId === user.id) {
        const hostBadge = document.createElement("span");
        hostBadge.textContent = "Host";
        hostBadge.classList.add("badge");

        playerElement.setAttribute("data-host", "true");
        playerElement.appendChild(hostBadge);
    }

    listOfPlayers.appendChild(playerElement);

    if (listOfPlayers.getAttribute("hidden")) {
        listOfPlayers.removeAttribute("hidden");
    }
}

export function updateHost(hostId: string) {
    const previousHost = document.querySelector("[data-host='true']");

    if (previousHost) {
        previousHost.removeAttribute("data-host");
        previousHost.querySelector(".badge")?.remove();

        if (previousHost.id.split("-")[1] === STATE.user?.id) {
            document.body.dataset.host = "false";
        }
    }

    const playerElement = document.getElementById(`player-${hostId}`);

    if (!playerElement) {
        console.error(
            "Received invalid user ID or user's info is not in the list of players in the DOM",
        );
        return;
    }

    const hostBadge = document.createElement("span");
    hostBadge.textContent = "Host";
    hostBadge.classList.add("badge");

    playerElement.setAttribute("data-host", "true");
    playerElement.appendChild(hostBadge);

    if (hostId === STATE.user?.id) {
        document.body.dataset.host = "true";
    }
}
