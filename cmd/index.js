import napi from '@workspace/napi_sandbox'

// napi.countAsync("Rust 1", (...a) => console.log(a))
// napi.countAsync("Rust 2", (...a) => console.log(a))

// setTimeout(async () => {
//   for (let i = 0; i < 4; i++) {
//     console.log(['JS 1', i])
//     await new Promise(res => setTimeout(res, 200))
//   }
// })

// console.log('run immediately')
// console.log('j1')
napi.foo(() => {
  console.log(globalThis['__neon_root_cache'])
  console.log('JS called')
}).then(v => console.log('done', v))

await new Promise((res) => setTimeout(res, 1000))
console.log(globalThis['__neon_root_cache'])

// console.log('j2')
