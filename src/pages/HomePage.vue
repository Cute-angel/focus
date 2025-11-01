<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

//import { window_start, openSpotlight } from "../api";

const greetMsg = ref("");
const name = ref("");
const text = ref("");
const intput_text = ref("");

async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    // greetMsg.value = await invoke("greet", { name: name.value });
    await window_start("settings", "/settings");
    console.log("11");
}

async function getText() {
    text.value = await invoke("set_text", { text: intput_text.value });
}

watch(intput_text, getText);

// 打开 Spotlight 窗口
async function openSpotlightWindow() {
    try {
        console.log("点击按钮，正在打开 Spotlight 窗口...");
        await openSpotlight();
        console.log("Spotlight 窗口已打开");
    } catch (error) {
        console.error("打开 Spotlight 窗口失败:", error);
    }
}
</script>

<template>
    <main class="container">
        <h1>Welcome to Tauri + Vue</h1>



        <form class="row" @submit.prevent="greet">
            <input id="greet-input" v-model="name" placeholder="Enter a name..." />
            <button type="submit">Greet</button>
        </form>
        <p>{{ greetMsg }}</p>

        <p><input type="text" v-model="intput_text"></p>
        <p> {{ text }}</p>
        <button class="btn w-64 rounded-full" @click="openSpotlightWindow">Button</button>
    </main>
</template>

<style scoped></style>