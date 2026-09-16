// Differential oracle fixture generator.
//
// Drives the vendor's own OLD (app.js, cid 102) and NEW (home/index-BTVblIUr.js, cid 62)
// protocol code, sealed in this sandbox, through a broad matrix of operations, and records the
// exact bytes each vendor call sends to `navigator.hid`. Nothing here re-implements the codec:
// every byte comes from evaluating the vendor's own extracted functions (lib_old.mjs / lib_new.mjs
// document exactly which region, by anchor text re-derived from the bundle every run).
//
// Output: a JSON fixture at $1 (the /out path), consumed by
// crates/hyperpace-protocol/tests/oracle.rs.
import fs from "node:fs";
import { buildOldSandbox, sha256 } from "./lib_old.mjs";
import { buildNewSandbox } from "./lib_new.mjs";

const OUT_PATH = process.argv[2];
if (!OUT_PATH) throw new Error("usage: generate.mjs <out.json>");

const APP_JS = "/vendor/app.js";
const NEW_JS = "/vendor/home/index-BTVblIUr.js";

const operations = [];

function toBytes16(bytes) {
  if (bytes.length !== 16) throw new Error(`expected 16 bytes, got ${bytes.length}`);
  return bytes;
}

function record(name, category, source, input, address, frames) {
  operations.push({
    name,
    category,
    source,
    input,
    address,
    frames: frames.map((f) => ({ reportId: f.reportId, bytes: toBytes16(f.bytes) })),
  });
}

// ---------------------------------------------------------------------------
// OLD bundle (primary oracle, cid 102 / mid 1)
// ---------------------------------------------------------------------------
{
  const { sandbox, captured } = buildOldSandbox(APP_JS);
  const run = async (name, category, input, address, fn) => {
    captured.length = 0;
    await fn();
    record(name, category, "old", input, address, captured.slice());
  };

  // Sensor table: the OLD cfg.json / sensor.json pairing (two ranges, maxDpi 40000).
  const OLD_TABLE = {
    range: [
      { min: 100, max: 30000, step: 50, DPIex: 0 },
      { min: 30100, max: 60000, step: 100, DPIex: 17 },
    ],
  };

  // -- command-only requests (section 5) --------------------------------------------------
  const commandOnly = [
    ["DeviceOnLine", 3, async () => sandbox.Le()],
    ["BatteryLevel", 4, async () => sandbox.Me(sandbox.te.BatteryLevel)],
    ["GetPairState", 6, async () => sandbox.Ee()],
    ["GetCurrentConfig", 14, async () => sandbox.Ne()],
    ["ReadVersionID", 18, async () => sandbox.Ue()],
    ["GetLongRangeMode", 23, async () => sandbox.ze()],
    ["GetDongleVersion", 29, async () => sandbox.Ye()],
  ];
  for (const [label, cmd, fn] of commandOnly) {
    await run(`command_only_${label}`, "command_only", { command: cmd }, 0, fn);
  }

  // -- scalar writes (section 7.1, offsets section 6.1) ------------------------------------
  const pollingRates = [125, 250, 500, 1000, 2000, 4000, 8000];
  for (const hz of pollingRates) {
    await run(`scalar_report_rate_${hz}`, "scalar", { hz }, 0, () => sandbox.es.Set_MS_ReportRate(hz));
  }
  for (const n of [1, 2, 3, 4, 5, 6, 7, 8]) {
    await run(`scalar_max_dpi_stage_${n}`, "scalar", { value: n }, 2, () => sandbox.es.Set_MS_MaxDPI(n));
  }
  for (const stage of [0, 1, 3, 7]) {
    await run(`scalar_current_dpi_${stage}`, "scalar", { value: stage }, 4, () =>
      sandbox.es.Set_MS_CurrentDPI(stage));
  }
  for (const ms of [0, 1, 8, 15]) {
    await run(`scalar_debounce_${ms}`, "scalar", { value: ms }, 169, () =>
      sandbox.es.Set_MS_DebounceTime(ms));
  }
  for (const lod of [1, 2, 3]) {
    await run(`scalar_lod_${lod}`, "scalar", { value: lod }, 10, () => sandbox.es.Set_MS_LOD(lod));
  }
  for (const v of [0, 1]) {
    await run(`scalar_motion_sync_${v}`, "scalar", { value: v }, 171, () => sandbox.es.Set_MS_MotionSync(v));
    await run(`scalar_angle_${v}`, "scalar", { value: v }, 175, () => sandbox.es.Set_MS_Angle(v));
    await run(`scalar_ripple_${v}`, "scalar", { value: v }, 177, () => sandbox.es.Set_MS_Ripple(v));
    await run(`scalar_moving_off_${v}`, "scalar", { value: v }, 179, () =>
      sandbox.es.Set_MS_MovingOffState(v));
    await run(`scalar_perf_state_${v}`, "scalar", { value: v }, 181, () =>
      sandbox.es.Set_MS_PerformanceState(v));
    await run(`scalar_sensor_mode_${v}`, "scalar", { value: v }, 185, () => sandbox.es.Set_MS_SensorMode(v));
    await run(`scalar_light_power_save_${v}`, "scalar", { value: v }, 94, () =>
      sandbox.es.Set_MS_LightPowerSave(v));
  }
  for (const code of [1, 3, 6, 12, 30, 60, 90]) {
    await run(`scalar_sleep_${code}`, "scalar", { value: code }, 173, () => sandbox.es.Set_MS_LightOffTime(code));
    await run(`scalar_perf_timeout_${code}`, "scalar", { value: code }, 183, () =>
      sandbox.es.Set_MS_PerformanceTime(code));
  }

  // -- DPI indicator (section 7.6) ---------------------------------------------------------
  for (const mode of [0, 1, 2]) {
    sandbox.ve.mouseCfg.dpiEffect.state = sandbox.le; // off, so mode-set also flips state on
    await run(`dpi_indicator_mode_${mode}`, "scalar", { value: mode }, 76, () =>
      sandbox.es.Set_MS_DPILightMode(mode));
  }
  for (const level of [1, 3, 5, 8, 10]) {
    await run(`dpi_indicator_brightness_${level}`, "scalar_computed", { level }, 78, () =>
      sandbox.es.Set_MS_DPILightBrightness(level));
  }
  for (const speed of [0, 4, 9, 255]) {
    await run(`dpi_indicator_speed_${speed}`, "scalar", { value: speed }, 80, () =>
      sandbox.es.Set_MS_DPILightSpeed(speed));
  }
  sandbox.ve.mouseCfg.dpiEffect.state = sandbox.oe;
  await run("dpi_indicator_off", "scalar", { value: 0 }, 82, () => sandbox.es.Set_MS_DPILightOff());

  // -- lighting (section 7.7): full struct + separate on/off scalar -----------------------
  const lightingCases = [
    { mode: 3, color: "rgb(255, 0, 0)", speed: 8, brightness: 3 }, // doc's worked example
    { mode: 0, color: "rgb(0, 0, 0)", speed: 0, brightness: 0 },
    { mode: 1, color: "rgb(0, 255, 128)", speed: 9, brightness: 9 },
    { mode: 5, color: "rgb(18, 52, 86)", speed: 2, brightness: 7 },
  ];
  for (const [i, c] of lightingCases.entries()) {
    sandbox.ve.mouseCfg.lightEffect.mode = c.mode;
    sandbox.ve.mouseCfg.lightEffect.color = c.color;
    sandbox.ve.mouseCfg.lightEffect.speed = c.speed;
    sandbox.ve.mouseCfg.lightEffect.brightness = c.brightness;
    await run(`lighting_struct_${i}`, "lighting_struct", c, 160, () => sandbox.kt());
  }
  sandbox.ve.mouseCfg.lightEffect.state = sandbox.le;
  await run("light_on", "scalar", { value: 1 }, 167, () => sandbox.es.Set_MS_LightMode(3));
  sandbox.ve.mouseCfg.lightEffect.state = sandbox.oe;
  await run("light_off", "scalar", { value: 0 }, 167, () => sandbox.es.Set_MS_LightMode(0));

  // -- DPI value + color, both model tables (section 7.4, 7.5) ----------------------------
  sandbox.ve.mouseCfg.sensor.cfg = OLD_TABLE;
  const oldDpis = [
    100, 150, 400, 800, 1600, 3200, 4000, 8000, 12000, 20000, 29950, 30000, // low range
    30100, 30200, 31000, 40000, 50000, 60000, // high range
  ];
  for (const stage of [0, 3, 7]) {
    for (const dpi of oldDpis) {
      await run(`dpi_value_old_stage${stage}_${dpi}`, "dpi_value", { stage, dpi, table: "old" },
        12 + 4 * stage, () => sandbox.es.Set_MS_DPIValue(stage, dpi));
    }
  }
  const NEW_TABLE_OLDSRC = { range: [{ min: 50, max: 30000, step: 50, DPIex: 0 }] };
  sandbox.ve.mouseCfg.sensor.cfg = NEW_TABLE_OLDSRC;
  const newDpis = [50, 100, 400, 800, 1600, 3200, 8000, 16000, 24000, 29950, 30000, 30100, 32000];
  for (const stage of [0, 2, 5]) {
    for (const dpi of newDpis) {
      await run(`dpi_value_new_stage${stage}_${dpi}`, "dpi_value", { stage, dpi, table: "new" },
        12 + 4 * stage, () => sandbox.es.Set_MS_DPIValue(stage, dpi));
    }
  }
  sandbox.ve.mouseCfg.sensor.cfg = OLD_TABLE;
  const colors = ["rgb(255, 0, 0)", "rgb(0, 255, 0)", "rgb(0, 0, 255)", "rgb(18, 52, 86)"];
  for (const stage of [0, 4, 7]) {
    for (const [i, color] of colors.entries()) {
      await run(`dpi_color_stage${stage}_${i}`, "dpi_color", { stage, color }, 44 + 4 * stage, () =>
        sandbox.es.Set_MS_DPIColor(stage, color));
    }
  }

  // -- buttons: KeyFunction record (section 8.1, 8.2) --------------------------------------
  const buttonCases = [
    { kind: 0, param: 0 },
    { kind: 1, param: 0x0100 },
    { kind: 1, param: 0x0200 },
    { kind: 1, param: 0x0400 },
    { kind: 1, param: 0x0800 },
    { kind: 1, param: 0x1000 },
    { kind: 2, param: 0x0100 },
    { kind: 2, param: 0x0200 },
    { kind: 2, param: 0x0300 },
    { kind: 3, param: 0x0100 },
    { kind: 3, param: 0x0200 },
    { kind: 4, param: (10 << 8) | 0 },
    { kind: 4, param: (255 << 8) | 3 },
    { kind: 5, param: 0 },
    { kind: 6, param: (0 << 8) | 1 },
    { kind: 6, param: (5 << 8) | 254 },
    { kind: 6, param: (2 << 8) | 255 },
    { kind: 7, param: 0 },
    { kind: 8, param: 0x1234 },
    { kind: 9, param: 0xffff },
    { kind: 10, param: 0 },
    { kind: 11, param: 0x00ff },
    { kind: 200, param: 0xabcd },
  ];
  for (const idx of [0, 3, 5]) {
    for (const [i, b] of buttonCases.entries()) {
      await run(`button_idx${idx}_case${i}`, "button", { index: idx, kind: b.kind, param: b.param },
        96 + 4 * idx, () => sandbox.es.Set_MS_KeyFunction(idx, { type: b.kind, param: b.param }));
    }
  }

  // -- media keys (section 8.5) -------------------------------------------------------------
  const mediaUsages = [0x00e9, 0x00ea, 0x00cd, 0x00b6, 0x00b5, 0x006f, 0x0070, 0x022a];
  for (const idx of [0, 2, 5]) {
    for (const usage of mediaUsages) {
      const hex = "0x" + usage.toString(16).padStart(4, "0");
      await run(`media_idx${idx}_${hex}`, "media", { index: idx, usage }, 256 + 32 * idx, () =>
        sandbox.es.Set_MS_Multimedia(idx, hex));
    }
  }

  // -- shortcut keystrokes / chords (section 8.4, 8.6) --------------------------------------
  const chordCases = [
    ["A"], ["Enter"], ["LCtrl", "A"], ["LCtrl", "LShift", "Z"], ["LWin", "RAlt", "F1"],
    ["LCtrl", "LShift", "LAlt", "LWin", "Space"],
  ];
  for (const idx of [0, 1, 4]) {
    for (const [i, chord] of chordCases.entries()) {
      await run(`shortcut_idx${idx}_case${i}`, "shortcut", { index: idx, chord }, 256 + 32 * idx, () =>
        sandbox.es.Set_MS_ShortcutKey(idx, chord));
    }
  }

  // -- macros: full slot write (section 8.7) -------------------------------------------------
  const macroCases = [
    { name: "hi", events: [{ press: true, kind: 1, value: 4, delay: 10 }] },
    {
      name: "combo",
      events: [
        { press: true, kind: 0, value: 1, delay: 5 },
        { press: true, kind: 1, value: 4, delay: 10 },
        { press: false, kind: 1, value: 4, delay: 10 },
        { press: false, kind: 0, value: 1, delay: 0 },
      ],
    },
    {
      name: "max thirty chars here!!",
      events: Array.from({ length: 20 }, (_, i) => ({
        press: i % 2 === 0,
        kind: 1,
        value: 4 + (i % 26),
        delay: 10 + i,
      })),
    },
    {
      name: "seventy",
      events: Array.from({ length: 70 }, (_, i) => ({
        press: i % 2 === 0,
        kind: i % 3 === 0 ? 4 : 1,
        value: (i * 7) % 256,
        delay: (i * 13) % 1000,
      })),
    },
  ];
  sandbox.ve.mouseCfg.macros = Array.from({ length: 6 }, () => ({ name: "", contexts: [] }));
  for (const idx of [0, 2, 5]) {
    for (const [i, m] of macroCases.entries()) {
      const contexts = m.events.map((e) => ({
        status: e.press ? 0 : 1,
        type: e.kind,
        value: e.value,
        delay: e.delay,
      }));
      await run(`macro_full_idx${idx}_case${i}`, "macro_full",
        { index: idx, name: m.name, events: m.events }, 768 + 384 * idx, () =>
          sandbox.es.Set_MS_Macro(idx, { name: m.name, contexts }));
    }
  }

  // -- flash reads (section 7.10, 4.9) -------------------------------------------------------
  const readCases = [
    [0, 10], [10, 10], [250, 6], [0, 1], [169, 2], [768, 10], [1150, 2], [16374, 10],
  ];
  for (const [addr, len] of readCases) {
    await run(`flash_read_${addr}_${len}`, "flash_read", { address: addr, len }, addr, () =>
      sandbox.et(addr, len));
  }

  // -- long range (section 7.9) --------------------------------------------------------------
  for (const on of [true, false]) {
    await run(`long_range_${on}`, "long_range", { on }, 0, () => sandbox.es.Set_Device_LongDistance(on ? 1 : 0));
  }

  // -- profile (section 10.2) ------------------------------------------------------------------
  for (const idx of [0, 1, 2, 3]) {
    await run(`profile_${idx}`, "profile", { index: idx }, 0, () => sandbox.es.Set_Device_Profile(idx));
  }

  // -- receiver light (section 10.5, OLD only) -------------------------------------------------
  // Color_To_Buffer (section 7.5) understands only "rgb(r, g, b)" strings and silently returns
  // black for anything else, including hex: this applies to every color setter, receiver light
  // included, so the test input must use the same format a real color-picker component sends.
  const receiverCases = [
    { mode: 0, color: "rgb(255, 0, 0)", speed: 5, brightness: 5, time: 1 },
    { mode: 3, color: "rgb(0, 255, 128)", speed: 9, brightness: 1, time: 30 },
  ];
  for (const [i, r] of receiverCases.entries()) {
    sandbox.ve.dongleLight.mode = r.mode;
    sandbox.ve.dongleLight.color = r.color;
    sandbox.ve.dongleLight.speed = r.speed;
    sandbox.ve.dongleLight.brightness = r.brightness;
    sandbox.ve.dongleLight.time = r.time;
    await run(`receiver_light_${i}`, "receiver_light", r, 0, () => sandbox.Ge());
  }

  // -- config file import clamp bug (section 11.2) ---------------------------------------------
  // openFile's DPI-stage clamp writes c[3] and c[5] using the vendor's own (buggy) formula.
  // maxDpiCount matches the real vendor cfg.json#mouse[0].cfg[0].dpis.length (6). The stage count
  // (6) is chosen to sit exactly at that ceiling, so the count clamp branch is a no-op and stays
  // identical on both sides; only the current-stage branch fires, isolating the check-byte bug
  // from Hyperpace's separate, deliberate choice to clamp the count itself to 8 (the memory map's
  // real slot capacity) rather than the vendor UI's model-specific maxDpiCount.
  {
    let capturedFlash = null;
    sandbox.es.Write_Mouse_Flash = async (c) => {
      capturedFlash = Array.from(c);
    };
    const fakeThis = { maxDpiCount: 6, $refs: { fileInput: { value: "" } } };
    const fileBytes = new Uint8Array(256 + 64);
    fileBytes[2] = 6; // stage count: at the vendor UI's ceiling, so its clamp is a no-op
    fileBytes[3] = 0x55 - 6; // already the correctly-paired complement
    fileBytes[4] = 7; // current stage: over maxDpiCount - 1 (5), so the buggy clamp fires
    fileBytes[5] = 0x55 - 7; // vendor's own correctly-paired complement, before the bug clobbers it
    const magic = new TextEncoder().encode("Compx Inc");
    fileBytes.set(magic, 256);
    const type = new TextEncoder().encode("mouse");
    fileBytes.set(type, 256 + 32);
    const sensor = new TextEncoder().encode("3950");
    fileBytes.set(sensor, 256 + 48);
    class FakeFileReader {
      set onload(fn) { this._onload = fn; }
      readAsArrayBuffer(file) {
        queueMicrotask(() => this._onload({ target: { result: file._bytes.buffer } }));
      }
    }
    const savedFileReader = sandbox.FileReader;
    sandbox.FileReader = FakeFileReader;
    const fakeEvent = { target: { files: [{ _bytes: fileBytes }] } };
    await sandbox.openFile.call(fakeThis, fakeEvent);
    sandbox.FileReader = savedFileReader;
    if (!capturedFlash) throw new Error("config import clamp test: openFile did not reach Write_Mouse_Flash");
    operations.push({
      name: "config_import_clamp_bug",
      category: "config_import_clamp",
      source: "old",
      input: { maxDpiCount: 6, rawStageCount: 6, rawCurrentStage: 7 },
      address: 2,
      frames: [],
      note: "vendor bytes 2..6 after openFile's clamp, not a sendReport frame",
      vendorClampedBytes: capturedFlash.slice(2, 6), // [count, countCheck, stage, stageCheck]
    });
  }
}

// ---------------------------------------------------------------------------
// NEW bundle (secondary oracle, cid 62 / mid 1): checksum + DPI encode under its own table.
// ---------------------------------------------------------------------------
{
  const { sandbox, captured } = buildNewSandbox(NEW_JS);
  sandbox.A.type = "mouse"; // A.type defaults to "keyboard" in /home; force the mouse class bit off
  sandbox.A.mouseCfg.sensor.cfg = { range: [{ min: 50, max: 30000, step: 50, DPIex: 0 }] };

  const run = async (name, category, input, address, fn) => {
    captured.length = 0;
    await fn();
    record(name, category, "new", input, address, captured.slice());
  };

  const newDpis = [50, 100, 800, 1600, 8000, 16000, 29950, 30000, 30100, 32000];
  for (const stage of [0, 3, 5]) {
    for (const dpi of newDpis) {
      await run(`new_dpi_value_stage${stage}_${dpi}`, "dpi_value", { stage, dpi, table: "new" },
        12 + 4 * stage, () => sandbox.Ah(stage, dpi));
    }
  }

  // Checksum function, standalone: independent confirmation that both bundles compute the same
  // (0x55 - sum) building block the frame checksum and every struct check byte derive from.
  const checksumSamples = [
    Uint8Array.of(3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239),
    Uint8Array.of(7, 0, 0, 169, 2, 8, 0x4d, 0, 0, 0, 0, 0, 0, 0, 0, 239),
    Uint8Array.of(8, 0, 0, 250, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239),
  ];
  for (const [i, frame] of checksumSamples.entries()) {
    operations.push({
      name: `new_checksum_${i}`,
      category: "checksum_fn",
      source: "new",
      input: { head: Array.from(frame.slice(0, 15)) },
      address: 0,
      frames: [],
      checksumResult: sandbox.nt(frame),
    });
  }
}

const fixture = {
  meta: {
    generated_at: new Date().toISOString(),
    bundles: {
      old: { path: "app.js", sha256: sha256(APP_JS), model: "cid 102, mid 1 (OLD, production mouse page)" },
      new: {
        path: "home/index-BTVblIUr.js",
        sha256: sha256(NEW_JS),
        model: "cid 62, mid 1 (NEW, device chooser)",
      },
    },
    how_to_regenerate:
      "make oracle (runs scripts/oracle.sh, which reruns this generator sealed under bubblewrap)",
    operation_count: operations.length,
  },
  operations,
};

fs.writeFileSync(OUT_PATH, JSON.stringify(fixture, null, 2));
console.log(`wrote ${operations.length} operations to ${OUT_PATH}`);

// A profile switch (Set_Device_Profile) triggers the vendor's own full-flash re-read, which arms
// a real 5 s battery-poll interval (section 10.2, section 9.6); nothing above needs it once the
// fixture is written, so exit explicitly instead of waiting for handles that outlive the script.
process.exit(0);
