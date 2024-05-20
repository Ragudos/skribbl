import { initializeToast } from "toastmynuts";

export const toast = initializeToast({
    richColors: true,
    theme: "light",
    position: {
        x: "right",
        y: "bottom",
    },
	toastDuration: 2500
});
