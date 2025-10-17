<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

import { window_start } from "./api";



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

</script>

<template>
  <main class="container">
    <h1>Welcome to Tauri + Vue</h1>

    <div class="row">
      <a href="https://vite.dev" target="_blank">
        <img src="/vite.svg" class="logo vite" alt="Vite logo" />
      </a>
      <a href="https://tauri.app" target="_blank">
        <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
      </a>
      <a href="https://vuejs.org/" target="_blank">
        <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
      </a>
    </div>
    <p>Click on the Tauri, Vite, and Vue logos to learn more.</p>

    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <button type="submit">Greet</button>
    </form>
    <p>{{ greetMsg }}</p>

    <p><input type="text" v-model="intput_text"></p>
    <p> {{ text }}</p>
    <button class="btn w-64 rounded-full" @click="greet">Button</button>
  </main>
</template>

<style scoped></style>