# SUMMARY: xueweiwujxw-2023-webkitwebprocess-high-cpu

## Claim
High WebKitWebProcess CPU on an Ubuntu 20.04 integrated-GPU machine was driven by the UI framework's CSS animations and transitions. Disabling them reduced usage substantially.

## Method
The reporter's own diagnosis, made by turning off MUI transitions.

## Result
Issue closed. Usage still peaked around 100% during page switches and menus.

## Evidence tier
3.

## Performance
Qualitative only.

## Correctness
2023, Tauri 1, an old WebKitGTK, and no controlled measurement.

## Relevance to hyperpace
Keep the Svelte UI free of continuous CSS animations (for example a pulsing battery icon) and heavy transitions. Prefer static rendering on Linux.
