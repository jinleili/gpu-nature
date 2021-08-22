import { createApp } from 'vue';
// doc: https://element3-ui.com/#/component/quickstart
import { ElRadio, ElSlider} from 'element3';
import 'element3/lib/theme-chalk/radio.css'
import 'element3/lib/theme-chalk/slider.css'

import './global.css';

import App from './App.vue';

const app = createApp(App);
app.use(ElRadio);
app.use(ElSlider);

app.mount('#app');

window.dispatchEventByName = function(eventName) {
    let elem = document.getElementById('canvas_container')
    if (elem == null) {
    return;
    }
    const event = new Event(eventName)
    elem.dispatchEvent(event)
}

var can_resize_canvas = true
window.canvas_resize_completed = function() {
    can_resize_canvas = true;
}

window.dispatch_resize_event = function() {
    can_resize_canvas = false;
    let elem = document.getElementById("canvas_container");
    if (elem != null) {
        elem.dispatchEvent(new Event("canvas_size_need_change"))    
    }
}

let pixelRation = window.devicePixelRatio || 1;
var preWidth = 0;
var preHeight = 0;
window.change_canvas_size = function() {
    let elem = document.getElementById("canvas_container");
    if (elem == null) {
        return;
    }
    let rect = elem.getBoundingClientRect();
    let canvas = elem.childNodes[0];
    if (rect.width != preWidth) {
        canvas.style.width = rect.width + "px";
        canvas.width = rect.width * pixelRation ;
        preWidth = rect.width;
    }
    if (rect.height != preHeight) {
        canvas.style.height = rect.height + "px";;
        canvas.height = rect.height * pixelRation;
        preHeight = rect.height;
    }
}

var timeOutFunctionId;
function window_resized() {
    clearTimeout(timeOutFunctionId);
    if (can_resize_canvas) {
        // Currently(2021/05/19), Firefox Nightly + winit(v0.24) change canvas size frequently will cause crash
        timeOutFunctionId = setTimeout(dispatch_resize_event, 800);
    } else {
        // Wait for the rust side to complete canvas resize
        console.log("user event: can_resize_canvas--false");
        timeOutFunctionId = setTimeout(window_resized, 800);
    }
}
window.onresize = window_resized;