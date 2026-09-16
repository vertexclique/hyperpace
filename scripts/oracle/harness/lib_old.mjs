// Loads the OLD driver bundle (app.js) and evaluates its self-contained device/protocol module
// (checksum, transaction, memory map, every Set_MS_* setter, DPI/keystroke/macro encoders) in an
// isolated vm context, driven by a stub navigator.hid. The vendor code is the thing that builds
// and sends every frame; this module only supplies the HID transport and the browser globals the
// bundle's top level touches, never a transcription of the protocol logic itself.
import fs from "node:fs";
import vm from "node:vm";
import crypto from "node:crypto";

export function sha256(path) {
  return crypto.createHash("sha256").update(fs.readFileSync(path)).digest("hex");
}

// Locate the module 6327 factory function (module,exports,require) without executing it, using
// the real V8 parser via Function.prototype.toString(), then slice out the pure protocol region
// and the config-file import handler by anchor text. Anchors are re-derived from the bundle on
// every run (never hardcoded byte offsets) so this stays correct if the bundle is rebuilt.
function moduleSource(appJsPath) {
  const src = fs.readFileSync(appJsPath, "utf8");
  const sandbox = { console };
  sandbox.self = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.window = sandbox;
  sandbox.navigator = { hid: { requestDevice: async () => [], getDevices: async () => [] } };
  sandbox.document = { createElement: () => ({ setAttribute() {}, appendChild() {}, style: {} }) };
  sandbox.localStorage = {
    _m: new Map(),
    getItem(k) { return this._m.has(k) ? this._m.get(k) : null; },
    setItem(k, v) { this._m.set(k, String(v)); },
  };
  vm.createContext(sandbox);

  const tailMarker = "a=s.O(a)})();";
  const idx = src.lastIndexOf(tailMarker);
  if (idx === -1) throw new Error("app.js: unrecognized bundle tail, cannot expose module map");
  const exposed =
    src.slice(0, idx) +
    "a=s.O(a);globalThis.__wpScope={s,e};})();" +
    src.slice(idx + tailMarker.length);
  vm.runInContext(exposed, sandbox, { filename: "app.js" });

  const { e: modules } = sandbox.__wpScope;
  const ids = Object.keys(modules);
  if (ids.length !== 1) {
    throw new Error(`app.js: expected exactly one own webpack module, found ${ids.length}`);
  }
  return modules[ids[0]].toString();
}

/** The pure protocol module: checksum, transaction, memory map, every Set_MS_* setter. */
export function extractProtocolRegion(fullSource) {
  const startAnchor = "const c={Escape:";
  const endAnchor = ",ts=function(){";
  const start = fullSource.indexOf(startAnchor);
  const end = fullSource.indexOf(endAnchor);
  if (start === -1 || end === -1 || end <= start) {
    throw new Error("app.js: protocol region anchors not found; bundle layout may have changed");
  }
  const region = fullSource.slice(start, end);
  if (!region.includes("var es={Request_Device:")) {
    throw new Error("app.js: extracted region is missing the expected device-api export object");
  }
  return region;
}

/** The config-file import handler (`openFile`), which carries the DPI clamp bug (section 11.2). */
export function extractImportHandler(fullSource) {
  const startAnchor = "openFile(e){";
  const endAnchor = ",handleImportClick(){";
  const start = fullSource.indexOf(startAnchor);
  const end = fullSource.indexOf(endAnchor, start);
  if (start === -1 || end === -1 || end <= start) {
    throw new Error("app.js: openFile anchors not found; bundle layout may have changed");
  }
  return `function ${fullSource.slice(start, end)}`;
}

/**
 * Build a fresh vm context with the OLD driver's protocol module (and optionally the config-file
 * import handler) evaluated at top level, plus a fake HID device wired in and visit mode turned
 * off so `ke()` actually sends. Returns the sandbox and the list of captured frames, which the
 * caller resets between operations.
 */
export function buildOldSandbox(appJsPath) {
  const full = moduleSource(appJsPath);
  const protocolRegion = extractProtocolRegion(full);
  const importHandler = extractImportHandler(full);

  const captured = [];

  const sandbox = {
    console,
    setTimeout,
    clearTimeout,
    setInterval,
    clearInterval,
    TextEncoder,
    TextDecoder,
    localStorage: {
      _m: new Map(),
      getItem(k) { return this._m.has(k) ? this._m.get(k) : null; },
      setItem(k, v) { this._m.set(k, String(v)); },
    },
  };
  sandbox.self = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.window = sandbox;
  vm.createContext(sandbox);

  vm.runInContext('"use strict";' + protocolRegion, sandbox, { filename: "old-protocol.js" });
  vm.runInContext(importHandler, sandbox, { filename: "old-import.js" });

  // The fake device: records every sendReport call, then answers with an echo of the sent frame
  // (satisfying the vendor's own acceptance rule, section 4.6) so `ke()` resolves on its first
  // attempt. DeviceOnLine (command 3) is additionally forced online, since every setter gates on
  // it and nothing in this isolated module ever populates a real reply.
  const fakeDevice = {
    opened: true,
    vendorId: 0x3554,
    async sendReport(reportId, data) {
      const bytes = Array.from(data);
      captured.push({ reportId, bytes });
      const reply = bytes.slice();
      reply[1] = 0; // status: decoded
      if (reply[0] === 3) reply[5] = 1; // DeviceOnLine: force online
      const replyBuf = Uint8Array.from(reply);
      const handler = sandbox.q && sandbox.q.oninputreport;
      if (handler) {
        await handler({ reportId, data: { buffer: replyBuf.buffer } });
      }
    },
  };
  sandbox.q = fakeDevice;
  sandbox.Se(); // installs the real vendor oninputreport dispatch onto q
  sandbox.Qt(false); // Set_Visit_Mode(false): leave visit mode, so ke() actually sends

  return { sandbox, captured };
}
