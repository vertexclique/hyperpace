; Installer hooks wired in via bundle.windows.nsis.installerHooks (see
; ../README.md). Windows needs no udev-equivalent permission step: the
; device enumerates as a standard USB HID collection and the OS grants the
; logged-in user access without a driver or admin step.
;
; The only custom behaviour here is on uninstall: Hyperpace keeps its macro
; library, profile snapshots and settings under %USERPROFILE%\.hyperpace
; (the same path on every OS, per the plan), which sits outside the install
; directory NSIS removes automatically. Ask before deleting it so an
; uninstall-then-reinstall does not silently lose the user's macros.
;
; This hook must not fire during an in-place upgrade: installing a newer
; version over an existing one makes the bundled installer silently
; uninstall the old one first with the /UPDATE flag, which the generated
; installer.nsi records in $UpdateMode (see tauri-bundler's own
; installer.nsi template, which gates its equivalent app-data cleanup the
; same way). Without this guard, every upgrade would pop up "remove your
; macros?" in the middle of what the user experiences as one update.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    IfFileExists "$PROFILE\.hyperpace\*.*" 0 hyperpace_no_data
      MessageBox MB_YESNO|MB_ICONQUESTION \
        "Also remove your Hyperpace macros, profiles and settings ($PROFILE\.hyperpace)?" \
        IDNO hyperpace_no_data
      RMDir /r "$PROFILE\.hyperpace"
    hyperpace_no_data:
  ${EndIf}
!macroend
