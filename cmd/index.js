import napi from '@workspace/napi_sandbox'

napi.countAsync("Rust 1", (...a) => console.log(a))
napi.countAsync("Rust 2", (...a) => console.log(a))

setTimeout(async () => {
  for (let i = 0; i < 4; i++) {
    console.log(['JS 1', i])
    await new Promise(res => setTimeout(res, 200))
  }
})

console.log('run immediately')