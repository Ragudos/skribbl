import {
    getCreatePrivateRoomBtn,
    getLobbyForm,
    getPlayBtn,
    submit,
} from "./lib/lobby";
import "./styles/index.css";

const playBtn = getPlayBtn();
const createPrivateRoomBtn = getCreatePrivateRoomBtn();
const lobbyForm = getLobbyForm();

lobbyForm.addEventListener("submit", playSubmit);
playBtn.addEventListener("click", playSubmit);
createPrivateRoomBtn.addEventListener("click", createSubmit);

async function playSubmit(evt: Event) {
    evt.preventDefault();

    playBtn.disabled = true;
    createPrivateRoomBtn.disabled = true;

    try {
        await submit("play");
    } finally {
        playBtn.disabled = false;
        createPrivateRoomBtn.disabled = false;
    }
}

async function createSubmit() {
    playBtn.disabled = true;
    createPrivateRoomBtn.disabled = true;

    try {
        await submit("create");
    } finally {
        playBtn.disabled = false;
        createPrivateRoomBtn.disabled = false;
    }
}
