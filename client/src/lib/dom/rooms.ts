export function showWaitingRoom() {
    const currentRoom = document.querySelector(".rooms[data-current='true']");

    if (currentRoom) {
        currentRoom.removeAttribute("data-current");
    }

    const waitingRoom = document.getElementById("waiting-room") as HTMLElement;

    waitingRoom.setAttribute("data-current", "true");
}

export function showLobbyRoom() {
    const currentRoom = document.querySelector(".rooms[data-current='true']");

    if (currentRoom) {
        currentRoom.removeAttribute("data-current");
    }

    const lobbyRoom = document.getElementById("lobby") as HTMLElement;

    lobbyRoom.setAttribute("data-current", "true");
}
