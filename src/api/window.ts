import { Window, WindowOptions } from "@tauri-apps/api/window";
import { Webview } from "@tauri-apps/api/webview";
import { WebviewOptions } from "@tauri-apps/api/webview";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export async function build_windows(
  label: string,

  windowOptions: WebviewOptions
) {
  const appwindow: Window = new Window(label, {});

  appwindow.once("tauri://created", async function () {
    console.log("window created");

    const webview: Webview = new Webview(appwindow, label, windowOptions);

    webview.once("tauri://created", async function () {});
  });

  appwindow.once("tauri://error", function (e) {
    console.error("An error occurred: ", e);
    // an error happened creating the window
  });
}

async function useCreateWebviewWindows(
  label: String,
  options:
    | (Omit<WebviewOptions, "width" | "height" | "x" | "y"> & WindowOptions)
    | undefined
): Promise<() => Window> {
  return () => {
    const window: Window = new WebviewWindow(label as string, options);
    return window;
  };
}

async function useCloseWebviewWindow(label: String): Promise<() => void> {
  const webview = await WebviewWindow.getByLabel(label as string);
  return () => {
    webview?.close();
  };
}

export { useCreateWebviewWindows, useCloseWebviewWindow };
