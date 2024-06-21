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
napi.start(() => console.log('JS called'))
// console.log('j2')
// await new Promise((res) => setTimeout(res, 1500))
// console.log('j3')
