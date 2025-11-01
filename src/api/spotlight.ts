import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

// 创建 Spotlight 窗口
export async function openSpotlight() {
  try {
    console.log("正在创建 Spotlight 窗口...");

    const spotlightWindow = new WebviewWindow("spotlight", {
      url: "/spotlight",
      width: 700,
      height: 500,
      center: true,

      decorations: false,
      transparent: true,
      focus: true,
      visible: true,
    });

    console.log("Spotlight 窗口创建成功");

    // 窗口失去焦点时自动关闭
    spotlightWindow.listen("tauri://blur", () => {
      console.log("Spotlight 窗口失去焦点，正在关闭...");
      //spotlightWindow.close();
    });

    spotlightWindow.once("tauri://created", () => {
      console.log("Spotlight 窗口已完全创建并显示");
    });

    spotlightWindow.once("tauri://error", (e) => {
      console.error("Spotlight 窗口创建失败:", e);
    });

    return spotlightWindow;
  } catch (error) {
    console.error("创建 Spotlight 窗口失败:", error);
    throw error;
  }
}
