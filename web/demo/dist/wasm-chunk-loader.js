/* eslint-disable no-console */
/**
 * WASM Chunk Loader - fetches chunked wasm files and reconstructs them
 * This loader is necessary because the selfhost server has a ~1MB upload limit,
 * so the 1.2MB wasm binary is split into 22 chunks of 60KB each.
 */

const CHUNKS = [
  "chunk.aa", "chunk.ab", "chunk.ac", "chunk.ad", "chunk.ae",
  "chunk.af", "chunk.ag", "chunk.ah", "chunk.ai", "chunk.aj",
  "chunk.ak", "chunk.al", "chunk.am", "chunk.an", "chunk.ao",
  "chunk.ap", "chunk.aq", "chunk.ar", "chunk.as", "chunk.at",
  "chunk.au", "chunk.av"
];

/**
 * Fetch and concatenate all wasm chunks into a single buffer
 */
async function fetchWasmBuffer() {
  console.log('Fetching rui wasm chunks...');
  const buffers = [];
  let totalBytes = 0;

  for (let i = 0; i < CHUNKS.length; i++) {
    const chunkName = CHUNKS[i];
    const chunkPath = `./rui_demo_bg.wasm.${chunkName}`;

    try {
      console.log(`Fetching chunk ${i + 1}/${CHUNKS.length}: ${chunkName}`);
      const response = await fetch(chunkPath);

      if (!response.ok) {
        throw new Error(`Failed to fetch ${chunkName}: ${response.status} ${response.statusText}`);
      }

      const buffer = await response.arrayBuffer();
      buffers.push(buffer);
      totalBytes += buffer.byteLength;
      console.log(`  Downloaded ${buffer.byteLength} bytes`);
    } catch (error) {
      console.error(`Error fetching chunk ${chunkName}:`, error);
      throw error;
    }
  }

  console.log(`All chunks fetched. Total size: ${totalBytes} bytes`);

  // Concatenate all buffers into a single Uint8Array
  const concatenated = new Uint8Array(totalBytes);
  let offset = 0;

  for (const buffer of buffers) {
    const chunk = new Uint8Array(buffer);
    concatenated.set(chunk, offset);
    offset += chunk.byteLength;
  }

  console.log('Wasm buffer reconstructed');
  return concatenated.buffer;
}

/**
 * Initialize wasm from the reconstructed buffer
 * This mirrors the initialization from rui_demo.js
 */
export async function initWasm() {
  console.log('Initializing wasm from chunked buffer...');

  try {
    // Fetch the reconstructed wasm buffer
    const wasmBuffer = await fetchWasmBuffer();

    // Import the rui_demo module
    // We need to load it in a way that allows us to pass the wasm buffer
    const { default: init } = await import('./rui_demo.js');

    // Initialize with the reconstructed buffer
    console.log('Calling wasm init with buffer...');
    await init(wasmBuffer);

    console.log('rui_demo started successfully');
    return { init };
  } catch (error) {
    console.error('Failed to initialize wasm:', error);
    console.error('Stack:', error.stack);
    throw error;
  }
}

// Auto-initialize when loaded
console.log('wasm-chunk-loader.js loaded');
initWasm().catch(err => {
  console.error('Fatal error in wasm initialization:', err);
  throw err;
});
