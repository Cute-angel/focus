<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

//import { window_start, openSpotlight } from "../api";


const text = ref("");
const intput_text = ref("");

async function getText() {
    text.value = await invoke("set_text", { text: intput_text.value });
}

watch(intput_text, getText);


</script>

<template>
    <main class="container">
    </main>
</template>

<style scoped></style>