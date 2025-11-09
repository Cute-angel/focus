import { useCloseWebviewWindow, useCreateWebviewWindows } from "./window";
import { openSpotlight } from "./spotlight";
import { invoke } from "@tauri-apps/api/core";

const useRunAction = (action_id: String, val: String) => {
  console.log(action_id);
  invoke("run_action", { id: action_id, val: val });
};

export {
  useCloseWebviewWindow,
  useCreateWebviewWindows,
  openSpotlight,
  useRunAction,
};
