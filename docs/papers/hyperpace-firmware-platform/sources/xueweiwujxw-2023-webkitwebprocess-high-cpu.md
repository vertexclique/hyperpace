URL: https://github.com/tauri-apps/tauri/issues/7183
Retrieved: 2026-09-15 (via `gh issue view 7183 --repo tauri-apps/tauri`)
Source type: GitHub issue (community report and self-diagnosis, tier 3); state CLOSED; opened 2023-06-12

Title: "[bug] WebKitWebProces high CPU usage"

## Verbatim, body (xueweiwujxw)

> After compiling on Ubuntu 20.04, I put the output on another Ubuntu 20.04 computer to run. The system on this computer is newly installed. The CPU usage of WebKitWebProcess is abnormally high, but it is normal on my local machine. My web code has not done much, only websocket communication once per second.
>
> Especially after the popup appears, the CPU usage will double directly.

## Verbatim, reporter follow-up (2023-06-13)

> It seems that the problem is caused by the Mui animation effect. After disabling it with the following code, the usage has significantly decreased, with the highest usage around 100%. This issue usually occurs during page switching or Menu rendering. However, both my local machine and the target machine are using integrated graphics, so it may be because the integrated graphics on my local machine are slightly stronger.

```ts
const tstheme = createTheme({
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        '*, *::before, *::after': {
          transition: 'none !import
```
(truncated in capture)
