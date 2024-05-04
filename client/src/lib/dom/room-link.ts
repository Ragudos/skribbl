import { STATE } from "../../state";

export function showRoomLink() {
    const roomLink = document.getElementById("room-link") as HTMLElement;

    if (!STATE.room) {
        roomLink.setAttribute("hidden", "");
        return;
    }

    const input = document.getElementById(
        "room-link-input",
    ) as HTMLInputElement;

    roomLink.removeAttribute("hidden");

    const roomLinkText = `${window.location.origin}?roomId=${STATE.room.id}`;

    input.value = roomLinkText;
}

export function hideShareRoomLink() {
    const roomLink = document.getElementById("room-link") as HTMLElement;
    roomLink.setAttribute("hidden", "");
}
