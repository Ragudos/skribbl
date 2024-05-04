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

    submit("play");
}

function createSubmit() {
    submit("create");
}
