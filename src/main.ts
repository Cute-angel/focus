import { createApp } from "vue";
import { createWebHistory, createRouter, Router } from "vue-router";

import App from "./App.vue";
import { path } from "@tauri-apps/api";

const routes = [
  {
    path: "/settings",
    component: () => import("./pages/SettingPage.vue"),
  },
];

const router: Router = createRouter({
  history: createWebHistory(),
  routes,
});

createApp(App).use(router).mount("#app");
