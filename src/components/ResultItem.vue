<template>
    <div class="flex flex-row hover:glass hover:shadow hover:rounded-field py-0.5"
        :class="{ 'bg-gray-200/30 shadow rounded-field': isSelect }">

        <div class="size-8 my-auto">

            <div v-html="icon" v-if="icon !== ''"></div>
        </div>




        <br>
        <div class="mx-2 flex-1 truncate">
            <h3 class="text-base-content text-2xl leading-none">{{ title }}</h3>
            <p class="text-base-content/60">{{ description }}</p>
        </div>
        <div class=" ml-auto">
            <actions-box :-action-list="actions" :-select-action="selectedAction" v-if="isShow" />
        </div>

    </div>
</template>

<script setup lang="ts">

import { computed, ComputedRef } from 'vue';
import ActionsBox from './ActionsBox.vue';
import { type Action } from './ActionsBox.vue'


const props = defineProps<{
    icon: String,
    title: String,
    description: String,
    actions: Array<Action>,
    isSelect: Boolean,
    selectedAction: Number
}>()

const isShow: ComputedRef<Boolean> = computed(() => {
    if (props.actions.length === 1 && props.actions[0].icon === "hide") {
        return false
    } else {
        return props.isSelect
    }
})


</script>

<style scoped></style>