<template>
    <div class="flex flex-row-reverse h-full ml-4 items-center" @keydown="handleKeydown">
        <div class="" v-for="(action, index) in reserve_list">
            <div class="tooltip box-border tooltip-bottom" :data-tip="action.tooltip"
                :key="reserve_list.length - 1 - index"
                :class="reserve_list.length - 1 - index === SelectAction ? 'tooltip-open' : ''">
                <button class="btn btn-square btn-outline mx-3 box-border outline-none border-none hover:border-2"
                    @click="useRunAction(action.id)"
                    :class="reserve_list.length - 1 - index === SelectAction ? 'btn-active' : ''">
                    <div v-html="action.icon" class="inline-block"></div>
                </button>
            </div>
        </div>
    </div>

</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRunAction } from '../api'

interface Action {
    id: String,
    tooltip: String,
    icon: String,

}

const props = defineProps<{
    ActionList: Array<Action>,
    SelectAction: Number
}>()



const handleKeydown = (event: KeyboardEvent) => {
    console.log(event)

}

const reserve_list = computed(() => {
    return [...props.ActionList].reverse()
})

//const nums: ComputedRef<Number> = computed(() => actionList.action.length);




export type { Action };
</script>

<style scoped>
button {
    background-color: rgba(red, green, blue, 0);
}
</style>