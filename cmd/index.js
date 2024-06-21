import napi from '@workspace/napi_sandbox'

console.log("[JS] 1")

napi.foo(() => console.log('[JS] Callback called'))
  .then(v => console.log('[JS] resolved:', v))

console.log('[JS] 2')

await new Promise((res) => setTimeout(res, 1000))

console.log('[JS] 3')
