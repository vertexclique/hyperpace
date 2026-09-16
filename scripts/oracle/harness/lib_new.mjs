// Same technique as lib_old.mjs, applied to the NEW driver bundle (home/index-BTVblIUr.js): the
// device/protocol region is a contiguous, self-contained slice (checksum, transaction, memory
// map, every Set_MS_* setter, plus the keyboard equivalents this task does not exercise). Used
// here only as the secondary oracle: checksum, and DPI encode under cid 62's native code.
import fs from "node:fs";
import vm from "node:vm";

export function extractProtocolRegion(fullSource) {
  const start = fullSource.indexOf("const Jn={Escape:");
  const end = fullSource.indexOf(",x6=Symbol()");
  if (start === -1 || end === -1 || end <= start) {
    throw new Error("index-BTVblIUr.js: protocol region anchors not found");
  }
  const region = fullSource.slice(start, end);
  if (!region.includes("Set_MS_DPIValue:Ah")) {
    throw new Error("index-BTVblIUr.js: extracted region is missing the expected export object");
  }
  return region;
}

export function buildNewSandbox(bundlePath) {
  const full = fs.readFileSync(bundlePath, "utf8");
  const region = extractProtocolRegion(full);

  const captured = [];
  const sandbox = { console, setTimeout, clearTimeout, setInterval, clearInterval, TextEncoder, TextDecoder };
  sandbox.self = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.window = sandbox;
  sandbox.navigator = { hid: {} };
  sandbox.localStorage = { getItem: () => null, setItem() {} };
  // Vue 3's reactive(): an identity stub. Reactivity is a UI-binding concern; every read and
  // write below goes through plain property access, which a Proxy-free object satisfies exactly
  // the same way, so this never changes what bytes the real setters below compute.
  sandbox.gt = (x) => x;
  vm.createContext(sandbox);

  vm.runInContext('"use strict";' + region, sandbox, { filename: "new-protocol.js" });

  const fakeDevice = {
    opened: true,
    async sendReport(reportId, data) {
      const bytes = Array.from(data);
      captured.push({ reportId, bytes });
      const reply = bytes.slice();
      reply[1] = 0;
      if (reply[0] === 3) reply[5] = 1; // DeviceOnLine
      const replyBuf = Uint8Array.from(reply);
      const handler = sandbox.ft && sandbox.ft.oninputreport;
      if (handler) {
        await handler({ reportId, data: { buffer: replyBuf.buffer } });
      }
    },
  };
  sandbox.ft = fakeDevice;
  sandbox.oi(); // installs the real vendor oninputreport dispatch onto ft
  sandbox.eg(false); // Set_Visit_Mode(false)

  return { sandbox, captured };
}
