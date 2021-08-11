
## Run on native

```sh
RUST_BACKTRACE=full cargo run --example nature
# or
RUST_BACKTRACE=full WGPU_POWER_PREF=high cargo run --example nature
```

## 优化
| 阶段| cpu | gpu|
|---|---|---|
| 前| 30ms| 16ms|
| 后| 20ms| 13.5ms|
| 只使用 16 长度 gradient 后| 19ms| 12.8ms|
