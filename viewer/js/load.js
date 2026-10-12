// Reading a file, parsing it with the module and reporting progress or errors.

export const CONFIRM_BYTES = 20 * 1024 * 1024;

/** True when the file is large enough that the user should confirm first. */
export function needsConfirmation(file) {
  return file.size > CONFIRM_BYTES;
}

const nextFrame = () => new Promise((resolve) => requestAnimationFrame(() => resolve()));

function megabytes(size) {
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * @param {object} options
 * @param {(bytes: Uint8Array) => string} options.parse  the module's parse
 * @param {HTMLElement} options.status   region that shows "Parsing…"
 * @param {HTMLElement} options.alert    role="alert" region for errors
 * @param {(loaded: {name: string, size: number, bytes: Uint8Array, view: object}) => void} options.onLoad
 * @param {() => void} options.onClear   drops the previous file's views
 */
export function createLoader({ parse, status, alert, onLoad, onClear }) {
  let ticket = 0;

  function fail(message) {
    onClear();
    alert.textContent = message;
    alert.hidden = false;
  }

  async function loadBytes(name, bytes) {
    const mine = ++ticket;
    alert.hidden = true;
    alert.textContent = '';
    status.textContent = 'Parsing…';
    status.hidden = false;
    await nextFrame();
    if (mine !== ticket) return;
    try {
      const view = JSON.parse(parse(bytes));
      if (mine !== ticket) return;
      onLoad({ name, size: bytes.length, bytes, view });
    } catch (err) {
      if (mine === ticket) fail(`${name}: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      if (mine === ticket) status.hidden = true;
    }
  }

  async function loadFile(file) {
    if (needsConfirmation(file)) {
      const ok = window.confirm(
        `${file.name} is ${megabytes(file.size)}. Parsing a file this large can take a while and use a lot of memory. Continue?`,
      );
      if (!ok) return;
    }
    const mine = ticket + 1;
    status.textContent = 'Reading…';
    status.hidden = false;
    let buffer;
    try {
      buffer = await file.arrayBuffer();
    } catch (err) {
      status.hidden = true;
      fail(`${file.name}: could not be read (${err instanceof Error ? err.message : String(err)})`);
      return;
    }
    if (ticket >= mine) return;
    await loadBytes(file.name, new Uint8Array(buffer));
  }

  return { loadFile, loadBytes };
}
