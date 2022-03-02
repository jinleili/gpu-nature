# gpu-nature

## DEMO

Running on FireFox Nightly
[https://jinleili.github.io/gpu-nature.html](https://jinleili.github.io/gpu-nature.html)

https://user-images.githubusercontent.com/1001342/120171869-0fdb0800-c235-11eb-8912-18a5b30e59cc.mov

## Run on native

```sh
cargo run --example {nature|particle}
```

## Run on iOS 

Please refer to [wgpu-on-app](https://github.com/jinleili/wgpu-on-app#ios)

## Run on browser

0th, reprocess shader.

> Reasons to handle shaders separately: when running the web environment, `wasm-pack` will monitor the changes of the `.rs` file and recompile for hot update, and the `build.rs` file will generate a new `web/generate_shader_map.rs` file, which will cause continuous loop compilation.

```sh
PREPROCESS_SHADER=true cargo build
```

First, install [Firefox Nightly](https://www.mozilla.org/en-US/firefox/channel/desktop/#nightly) and enable `Web API: WebGPU`. Or, install Chrome Canary/Dev and setting: `chrome://flags/#enable-unsafe-webgpu`

Second, install wasm-pack:

```sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

Third, compile to wasm and start run:

```sh
npm install
# only build
RUSTFLAGS=--cfg=web_sys_unstable_apis npm run build
# build then run on browser
RUSTFLAGS=--cfg=web_sys_unstable_apis npm start
```

Nhat-Phuong Tran, Myungho Lee, and Sugwon Hong, ["Performance Optimization of 3D Lattice Boltzmann Flow Solver on a GPU"](https://www.hindawi.com/journals/sp/2017/1205892/), Scientific Programming, 2017.

Martin Bauer, Ulrich Rüde, ["An improved lattice Boltzmann D3Q19 method based on an alternative equilibrium discretization"](https://arxiv.org/pdf/1803.04937.pdf), arXiv preprint arXiv:1803.04937, 2018.


## License

Released under the [GPLv3](https://github.com/MixinNetwork/desktop-app/blob/master/LICENSE) license.