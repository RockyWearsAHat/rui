// rui's WebGPU presenter: draws a recorded frame (src/gpu.rs) as instanced
// quads with src/gpu/shader.wgsl. Kept in JS rather than web-sys because the
// WebGPU bindings there sit behind an unstable cfg every consumer would have
// to set.
//
// The device is requested while this module loads (top-level await), so by
// the time the wasm runs it is either ready or known to be missing, and the
// backend can choose WebGPU or the 2D canvas synchronously. A page can hand
// in a device of its own as `globalThis.__ruiGpuDevice` to share one GPU
// with its other drawing.

const TIMEOUT_MS = 1500

async function requestDevice() {
  if (globalThis.__ruiGpuDevice) return globalThis.__ruiGpuDevice
  if (!globalThis.navigator?.gpu) return null
  const within = (p) => Promise.race([p, new Promise((r) => setTimeout(() => r(null), TIMEOUT_MS))])
  try {
    const adapter = await within(navigator.gpu.requestAdapter({ powerPreference: 'high-performance' }))
    if (!adapter) return null
    return await within(adapter.requestDevice())
  } catch {
    return null
  }
}

let device = globalThis.__ruiNoGpu ? null : await requestDevice()
let gpu = null // everything built by rui_gpu_attach

export function rui_gpu_available() {
  return !!device
}

const ATLAS = 2048

export function rui_gpu_attach(canvas, shader) {
  if (!device) return false
  try {
    const context = canvas.getContext('webgpu')
    if (!context) return false
    const format = navigator.gpu.getPreferredCanvasFormat()
    context.configure({ device, format, alphaMode: 'premultiplied' })
    const module = device.createShaderModule({ code: shader })
    const layout = device.createBindGroupLayout({
      entries: [
        { binding: 0, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
        { binding: 1, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'read-only-storage' } },
        { binding: 2, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'unfilterable-float' } },
        { binding: 3, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'unfilterable-float' } },
      ],
    })
    const pipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [layout] })
    const pipeline = (add) =>
      device.createRenderPipeline({
        layout: pipelineLayout,
        vertex: { module, entryPoint: 'vs' },
        fragment: {
          module,
          entryPoint: 'fs',
          targets: [{
            format,
            blend: add
              ? { color: { srcFactor: 'one', dstFactor: 'one' }, alpha: { srcFactor: 'one', dstFactor: 'one' } }
              : {
                  color: { srcFactor: 'one', dstFactor: 'one-minus-src-alpha' },
                  alpha: { srcFactor: 'one', dstFactor: 'one-minus-src-alpha' },
                },
          }],
        },
        primitive: { topology: 'triangle-strip' },
      })
    const atlas = device.createTexture({
      size: [ATLAS, ATLAS],
      format: 'r8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    })
    const blank = device.createTexture({
      size: [1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    })
    gpu = {
      context,
      layout,
      over: pipeline(false),
      add: pipeline(true),
      uniforms: device.createBuffer({ size: 16, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST }),
      shapes: null,
      capacity: 0,
      atlas,
      atlasView: atlas.createView(),
      blankView: blank.createView(),
      images: new Map(), // id → { texture, view }
      groups: new Map(), // image id → bind group, rebuilt when the shape buffer grows
    }
    device.lost?.then(() => { gpu = null; device = null })
    // A validation error otherwise leaves a silently blank canvas.
    device.addEventListener?.('uncapturederror', (e) => console.error('rui: WebGPU', e.error?.message ?? e))
    module.getCompilationInfo?.().then((info) => {
      for (const m of info.messages) if (m.type === 'error') console.error(`rui: shader ${m.lineNum}: ${m.message}`)
    })
    return true
  } catch (error) {
    console.error('rui: WebGPU attach failed', error)
    gpu = null
    return false
  }
}

export function rui_gpu_atlas(x, y, w, h, bytes) {
  if (!gpu) return
  device.queue.writeTexture({ texture: gpu.atlas, origin: [x, y] }, bytes, { bytesPerRow: w }, [w, h])
}

export function rui_gpu_image(id, w, h, bytes) {
  if (!gpu) return
  const texture = device.createTexture({
    size: [w, h],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
  })
  device.queue.writeTexture({ texture }, bytes, { bytesPerRow: w * 4 }, [w, h])
  gpu.images.set(id, { texture, view: texture.createView() })
  gpu.groups.delete(id)
}

export function rui_gpu_drop_image(id) {
  if (!gpu) return
  gpu.images.get(id)?.texture.destroy()
  gpu.images.delete(id)
  gpu.groups.delete(id)
}

function group(image) {
  let g = gpu.groups.get(image)
  if (g) return g
  g = device.createBindGroup({
    layout: gpu.layout,
    entries: [
      { binding: 0, resource: { buffer: gpu.uniforms } },
      { binding: 1, resource: { buffer: gpu.shapes } },
      { binding: 2, resource: gpu.atlasView },
      { binding: 3, resource: gpu.images.get(image)?.view ?? gpu.blankView },
    ],
  })
  gpu.groups.set(image, g)
  return g
}

// `shapes` is the flat instance list (28 floats each); `batches` is
// (first, count, add, image) per draw; `clear` is straight-alpha RGBA or empty.
export function rui_gpu_frame(width, height, shapes, batches, clear) {
  if (!gpu || width === 0 || height === 0) return false
  const bytes = Math.max(shapes.byteLength, 112)
  if (bytes > gpu.capacity) {
    gpu.shapes?.destroy()
    gpu.capacity = Math.max(bytes, gpu.capacity * 2, 64 * 1024)
    gpu.shapes = device.createBuffer({ size: gpu.capacity, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST })
    gpu.groups.clear()
  }
  if (shapes.byteLength) device.queue.writeBuffer(gpu.shapes, 0, shapes)
  device.queue.writeBuffer(gpu.uniforms, 0, new Float32Array([width, height, 0, 0]))

  const [r, g, b, a] = clear.length === 4 ? clear : [0, 0, 0, 0]
  const encoder = device.createCommandEncoder()
  const pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: gpu.context.getCurrentTexture().createView(),
      clearValue: { r: r * a, g: g * a, b: b * a, a },
      loadOp: 'clear',
      storeOp: 'store',
    }],
  })
  let bound = null
  for (let i = 0; i < batches.length; i += 4) {
    const [first, count, add, image] = [batches[i], batches[i + 1], batches[i + 2], batches[i + 3]]
    pass.setPipeline(add ? gpu.add : gpu.over)
    if (bound !== image) {
      pass.setBindGroup(0, group(image))
      bound = image
    }
    pass.draw(4, count, 0, first)
  }
  pass.end()
  device.queue.submit([encoder.finish()])
  return true
}
