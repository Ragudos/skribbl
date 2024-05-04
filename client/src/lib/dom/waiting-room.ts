export function showWaitingRoom() {
    const currentRoom = document.querySelector(".rooms[data-current='true']");

    if (currentRoom) {
        currentRoom.removeAttribute("data-current");
    }

    const waitingRoom = document.getElementById("waiting-room") as HTMLElement;

    waitingRoom.setAttribute("data-current", "true");
}
