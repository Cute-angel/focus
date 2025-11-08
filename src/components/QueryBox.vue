<template>
    <div class="box-border">
        <magnifying-glass-icon class="size-8 w-fit ml-4" />
        <input type="text" class="user_input h-fit w-full  px-2 py-4  text-2xl outline-0" v-model="query"
            placeholder="Enter your query..." @keyup="updateCursor" @click="updateCursor" ref="inputElement" />
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { MagnifyingGlassIcon } from "@heroicons/vue/24/outline";


const cursorPos = defineModel<Number>('cursorPos');
const query = defineModel<String>('query');
const inputElement = ref<HTMLElement>();

watch(query, (newQuery) => {
    console.log("Query changed:", newQuery);
});



const updateCursor = (e: Event) => {
    const inputElement = e.target as HTMLInputElement
    cursorPos.value = inputElement.selectionStart ?? 0
    console.log(cursorPos.value)
}


onMounted(() => {
    inputElement.value?.focus();
})
</script>

<style scoped>
div {
    display: flex;
    justify-content: center;
    align-items: center;
}
</style>