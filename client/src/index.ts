import {
    createPrivateRoomBtnListener,
    playBtnListener,
    playFormListener,
} from "./lobby";
import "./styles/index.css";

window.addEventListener("DOMContentLoaded", init);

function init() {
    playFormListener.listen();
    playBtnListener.listen();
    createPrivateRoomBtnListener.listen();
}
