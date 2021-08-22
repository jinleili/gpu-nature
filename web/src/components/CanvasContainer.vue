<template>
  <div id="canvas_container" @click="singleClick" />
</template>

<script>
import { defineComponent } from 'vue'

export default defineComponent({
    setup() {
        const singleClick = (e) => {
            let x = e.offsetX
            let y = e.offsetY
            console.log("click P: " + x + ", " + y)
        }
        return { singleClick }
    },
})
</script>
