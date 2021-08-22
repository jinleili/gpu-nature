<template>
  <div class="row">
    <div class="row_item0">
      Particle size (physical pixel):
    </div>
    <div class="row_item1">
      <step-number
        v-model="particleSize"
        :min="1"
        :max="5"
        :step="1" />
    </div>
  </div>
    
  <div class="row">
    <div class="row_item0">
      Particles count:
    </div>
    <div class="row_item1">
      <step-number
        v-model="particlesCount"
        :min="10000"
        :max="200000"
        :step="10000" />
    </div>
  </div>

  <div class="row">
    <div class="row_item0">
      Particle color:
    </div>
    <div class="row_item1">
      <select v-model="colorType" class="border_comp">
        <option v-for="option in colorOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>
  </div>
</template>

<script>
import {defineComponent, ref, watch, reactive, toRefs } from 'vue'
import StepNumber from './StepNumber.vue'

export default defineComponent({
    components: {
        StepNumber,
    },
    setup() {
        const particleSize = ref(2)
        localStorage.setItem('particle_size', particleSize.value)
        watch(particleSize, (newValue, preValue) => {
        localStorage.setItem('particle_size', newValue)
        dispatchEventByName('particle_size_changed')
        })

        const particlesCount = ref(30000)
        localStorage.setItem('particles_count', particlesCount.value)
        watch(particlesCount, (newValue, preValue) => {
        localStorage.setItem('particles_count', newValue)
        dispatchEventByName('particles_count_changed')
        })

        const colorType = ref('1')
        watch(colorType, (newValue, preValue) => {
        localStorage.setItem('color_type', newValue)
        dispatchEventByName('particle_color_changed')
        })
        const colorOptionsData = reactive({
        colorOptions: [
            {
            value: '0',
            label: 'Uniform',
            },
            {
            value: '1',
            label: 'Movement angle',
            },
            {
            value: '2',
            label: 'Speed',
            },
        ],
        })
        return {
            particlesCount,
            particleSize,
            colorType,
            ...toRefs(colorOptionsData),
        }
    },
})
</script>
