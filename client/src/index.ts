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

function playSubmit(evt: Event) {
    evt.preventDefault();

    playBtn.disabled = true;
    createPrivateRoomBtn.disabled = true;

    submit("play");
}

function createSubmit() {
    playBtn.disabled = true;
    createPrivateRoomBtn.disabled = true;

    submit("create");
}
