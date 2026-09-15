# SUMMARY: fabianlars-2023-tray-app-without-webview

## Claim
Hiding a window keeps the webview processes alive. To free them, close the webview window and create it again on demand from Rust; the app keeps running without windows.

## Method
An accepted maintainer answer on a GitHub discussion, plus a related maintainer comment explaining the process layout: the webview content process, the Rust process, and GPU and network helper processes.

## Result
Guidance only; no measurements.

## Evidence tier
2.

## Performance
Not measured. The implied trade-off is window re-creation latency and lost in-page JS state, against idle memory and CPU.

## Correctness
Written in the Tauri 1 era (2023), but the mechanism maps to Tauri 2 (`ExitRequested` plus `prevent_exit`, window destroy, and `WebviewWindowBuilder`). Whether WebKitGTK tears down all helper processes (network, GPU) when the last webview is destroyed was not verified.

## Relevance to hyperpace
hyperpace keeps all device state in Rust, so destroying the window on close is natural. The phase-rs NVIDIA report found that minimizing did not reduce idle CPU. Together they make destroy-on-hide the default candidate. An experiment is needed to measure re-open latency and leftover processes on each OS.
