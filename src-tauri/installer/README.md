# Data-preserving Windows installer

Based on the official Tauri CLI 2.12.0 NSIS template:
https://raw.githubusercontent.com/tauri-apps/tauri/tauri-cli-v2.12.0/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi

Upstream SHA-256 (LF UTF-8): dabed59013b1d78b879a1a85bc7f2eed2993b33a9a90cdabe5946de3d3950597

Local changes are restricted to removal of the app-data checkbox/show/leave functions and the complete conditional app-data deletion block, plus an explicit preservation message. All executable/resource/shortcut/registry removal and update/WebView2 behavior retain the pinned template. There is no app-data deletion command for silent/passive mode either. Tauri MIT/Apache notices are included in THIRD_PARTY_NOTICES.txt.

This uses Tauri's supported bundle.windows.nsis.template setting. When upgrading the CLI, review the matching upstream template and reapply only these changes; do not revert to the default destructive option. Run scripts/verify-installer.mjs, build the installer, then run that check with --generated. Static checks and successful compilation do not replace uninstall/reinstall testing on an independent Windows environment.
