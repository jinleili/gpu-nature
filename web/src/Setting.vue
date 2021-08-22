<template>
  <div id="control_panel">
    <el-radio v-model="fieldType" label="1">
      Programmable Field
    </el-radio>
    <el-radio v-model="fieldType" label="2">
      LBM Fluid
    </el-radio>
    <div class="divider" />
    <div id="pfProperties">
      <p>Animation type: &nbsp; </p>
      <div>
        <el-radio v-model="fieldAnimationType" label="1">
          Spiral
        </el-radio>
        <el-radio v-model="fieldAnimationType" label="2">
          Julia set
        </el-radio>
      </div>
    </div>

    <div id="lbmProperties">
      <p>Animation type: &nbsp; </p>
      <div>
        <el-radio v-model="fluidAnimationType" label="3">
          Poiseuille Flow
        </el-radio>
        <el-radio v-model="fluidAnimationType" label="4">
          Custom Touch Move 
        </el-radio>
        <div class="tips">
          Tips:
          <ul>
            <li
              v-for="item in tipsMsgList"
              :key="item.msg">
              {{ item.msg }}
            </li>
          </ul>
        </div>
        <button class="cornerBt" @click="onReset">
          {{ cleanBtTitle }}
        </button>
        <div class="divider" />
      </div>
      <div class="col">
        <div class="col_item0">
          Fluid viscosity:
        </div>
        <div class="col_item1">
          0.005
        </div>
        <div class="col_item2">
          <el-slider
            v-model="fluidViscosity"
            :min="0.005"
            :max="0.23"
            :step="0.005"
            label="" />
        </div>
        <div class="col_item3">
          0.20
        </div>
      </div>
    </div>

    <div class="divider" />

    <particles-parameter />
  </div>
</template>

<script>
import { ref, watch, computed } from 'vue'
import ParticlesParameter from './components/ParticlesParameter.vue'

export default {
  components: {
    ParticlesParameter,
  },
  // When setup is executed, the component instance has not been created yet.
  // So, will not have access to methods.
  setup() {
    const fieldType = ref('1')
    watch(fieldType, (newValue, preValue) => {
      let pfProperties = document.getElementById('pfProperties')
      let lbmProperties = document.getElementById('lbmProperties')
      var animation_ty;
      if (newValue === '1') {
        pfProperties.style.display = 'block'
        lbmProperties.style.display = 'none'
        animation_ty = fieldAnimationType.value;
      } else {
        pfProperties.style.display = 'none'
        lbmProperties.style.display = 'block'
        animation_ty = fluidAnimationType.value;
      }
      localStorage.setItem('field_animation_type', animation_ty)
      localStorage.setItem('field_type', newValue)
      dispatchEventByName('field_type')
    })

    const fieldAnimationType = ref('2')
    watch(fieldAnimationType, (newValue, preValue) => {
      localStorage.setItem('field_animation_type', newValue)
      dispatchEventByName('field_animation_type')
    })

    const fluidAnimationType = ref('3')
    watch(fluidAnimationType, (newValue, preValue) => {
      localStorage.setItem('field_animation_type', newValue)
      dispatchEventByName('field_animation_type')
    })

    const msg0 = { msg: 'Swipe on the canvas to apply external force.'}
    const msg1 = { msg:'Click on the canvas to add obstacle.'}
    const tipsMsgList = computed(() => {
      if (fluidAnimationType.value === '3') {
        return [msg0, msg1]
      } else {
        return [msg0]
      }
    })

    const cleanBtTitle = computed(() => {
      if (fluidAnimationType.value === '3') {
        return 'Clean Added Obstacle'
      } else {
        return 'Clean Canvas'
      }
    })

    const fluidViscosity = ref(0.02);
    watch(fluidViscosity, (newValue, preValue) => {
      localStorage.setItem('fluid_viscosity', newValue)
      dispatchEventByName('fluid_viscosity_changed')
    })

    return {
      fieldType,
      fieldAnimationType,
      fluidAnimationType,
      fluidViscosity,
      tipsMsgList,
      cleanBtTitle,
    }
  },
  methods: {
    onReset() {
      let elem = document.getElementById('canvas_container')
      // keep 'dislay: none;' element prioritized get changed value
      const event = new Event("canvas_reset")
      elem.dispatchEvent(event)
    }
  },
}
</script>


<style lang="scss">
@import 'global.scss';

#control_panel {
  width: 100%;
  margin: 0px;
  padding: 24px 16px 30px 16px;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}
#lbmProperties {
  display: none;
}
.el-col {
  line-height: 40px;
}

</style>
