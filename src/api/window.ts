import { Window } from "@tauri-apps/api/window";
import { Webview } from "@tauri-apps/api/webview";

export async function window_start(label: string, path: string) {
  const appwindow: Window = new Window(label, {});

  appwindow.once("tauri://created", async function () {
    console.log("window created");

    const webview: Webview = new Webview(appwindow, label, {
      url: path,
      x: 0,
      y: 0,
      width: 800,
      height: 600,
    });

    webview.once("tauri://created", async function () {});
  });

  appwindow.once("tauri://error", function (e) {
    console.error("An error occurred: ", e);
    // an error happened creating the window
  });
}
