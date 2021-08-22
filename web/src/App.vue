<template>
  <div id="col_container">
    <div id="mail_col"> 
      <canvas-container />
    </div>
    <div id="right_col"> 
      <setting />
    </div>
  </div>
</template>

<script>
import Setting from './Setting.vue';
import CanvasContainer from './components/CanvasContainer.vue'
export default {
  components: {
    'setting': Setting,
    CanvasContainer,
  },
}
</script>

<style lang="scss">
@import 'global.scss';

#canvas_container, #col_container {
  width: 100%; height: 100%; margin: 0px;  padding: 0px;
}
#col_container::after {
  display: block;
  clear: both;
}
#mail_col {
  width: 100%;
  background-color: $main-bg-color;
  color: #fff;
  padding: 0px;
  height: 100%;
}
#right_col {
  width: $toolbox-width;
  max-width: 90%;
  color: #bbb;  
  background-color: $toolbox-bg-color;
  position: absolute;
  top: 0px; right: 0px;
}
#right_col::after {
  display: block;
  clear: both;
}
@media screen and (min-width: 1000px) {
  #mail_col {
    padding-right: $toolbox-width;
  }
  #right_col {
    height: 100%;
    border-left: $border;
  }
} 
@media screen and (max-width: 999px) {
  #mail_col {
    margin: 0px;
  }
  #right_col {
      top: 12px; right: 12px;
      border: $border;
      border-radius: 8px;
  }
} 

  
</style>