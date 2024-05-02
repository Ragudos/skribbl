export function getPlayBtn() {
    return document.getElementById("play-btn") as HTMLButtonElement;
}

export function getCreatePrivateRoomBtn() {
    return document.getElementById(
        "create-private-room-btn",
    ) as HTMLButtonElement;
}

export function getLobbyForm() {
    return document.getElementById("lobby-form") as HTMLFormElement;
}

export function getDisplayName() {
    return (document.getElementById("display-name") as HTMLInputElement).value;
}

export function getRoomId() {
    return (document.getElementById("room-id") as HTMLInputElement).value;
}
