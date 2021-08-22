<template>
  <div class="counter border_comp">
    <span class="decrease" :class="{'isDisabled': minDisabled}" @click="decrease"> - </span>
    <label class="counter_display">{{ displayValue }}</label>
    <span class="increase" :class="{'isDisabled': maxDisabled}" @click="increase"> + </span>
  </div>
</template>
<script>
import { defineComponent, reactive, computed } from 'vue'

export default defineComponent({
    props: {
        step: { 
            type: Number,
            default: 1
        },
        min: { 
            type: Number,
            default: 0
        },
        max: { 
            type: Number,
            default: 10
        },
        modelValue: {
            type: Number,
            default: 0
        },
    },
    emits: ['update:modelValue'],
    setup(props, { emit }) {
        const data = reactive({ currentValue: props.modelValue })
        const displayValue = computed(() => {
            return data.currentValue
        })
        const minDisabled = computed(() => {
            return _decrease(props.modelValue) < props.min
        })
        const maxDisabled = computed(() => {
            return _increase(props.modelValue) > props.max
        })
        const numPrecision = computed(() => {
            const stepPrecision = getPrecision(props.step)
            return Math.max(getPrecision(props.modelValue), stepPrecision)
        })
        const toPrecision = (num, pre) => {
            if (pre === undefined) pre = numPrecision.value
            return parseFloat(
                Math.round(num * Math.pow(10, pre)) / Math.pow(10, pre) + '',
            )
        }
        const getPrecision = value => {
            if (value === undefined) return 0
            const valueString = value.toString()
            const dotPosition = valueString.indexOf('.')
            let precision = 0
            if (dotPosition !== -1) {
                precision = valueString.length - dotPosition - 1
            }
            return precision
        }
        const _increase = val => {
            const precisionFactor = Math.pow(10, numPrecision.value)
            // Solve the accuracy problem of JS decimal calculation by converting the value to integer.
            return toPrecision(
                (precisionFactor * val + precisionFactor * props.step) / precisionFactor,
            )
        }
        const _decrease = val => {
            const precisionFactor = Math.pow(10, numPrecision.value)
            return toPrecision(
                (precisionFactor * val - precisionFactor * props.step) / precisionFactor,
            )
        }
        const increase = () => {
            if (maxDisabled.value) return
            const value = props.modelValue || 0
            const newVal = _increase(value)
            setCurrentValue(newVal)
        }
        const decrease = () => {
            if (minDisabled.value) return
            const value = props.modelValue || 0
            const newVal = _decrease(value)
            setCurrentValue(newVal)
        }
        const setCurrentValue = newVal => {
            const oldVal = data.currentValue
            if (newVal >= props.max) newVal = props.max
            if (newVal <= props.min) newVal = props.min
            if (oldVal === newVal) return
            emit('update:modelValue', newVal)
            data.currentValue = newVal
        }
       return {displayValue, minDisabled, maxDisabled, increase, decrease} 
    },
})
</script>

<style lang="scss">
@import '../global.scss';

.counter {
    display: -webkit-flex; /* Safari */
    display: flex;
    flex-flow: row nowrap;
    justify-content: space-between;
    line-height: 36px; height: 36px; 
    text-align: center;
    border: $border;
    border-radius: $border-radius;
}
.counter_display {
    min-width: 60px;
}
.increase {
    border-left: $border; 
}
.decrease {
    border-right: $border; 
}
.increase, .decrease {
    width: 40px; 
    font-size: 22px;
    cursor: pointer;
}
</style>