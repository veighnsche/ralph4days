# macOS Dev Build and Install Checklists

## Maker Checklist (Build + Share)

1. On your Mac, build a macOS release of `ralph4days` (`just build` or your standard Tauri release command).
2. Launch the built app once locally to confirm it starts.
3. Package the app as a `.zip` (recommended for WhatsApp transfer).
4. Rename the artifact clearly, for example: `ralph4days-macos-dev-YYYY-MM-DD.zip`.
5. Send the zip to the client through WhatsApp.
6. Send the client installation checklist (below) in the same message.
7. Record the exact commit hash used for the build so feedback maps to a specific code version.

## Client Checklist (Install Unsigned Dev Build)

1. Download the `.zip` from WhatsApp on your Mac.
2. Unzip it to get `Ralph4Days.app`.
3. Drag `Ralph4Days.app` into `Applications`.
4. Try opening it once (double-click).
5. If blocked, right-click `Ralph4Days.app`, choose `Open`, then click `Open` again.
6. If still blocked, open `System Settings` > `Privacy & Security` and click `Open Anyway` for Ralph4Days.
7. If it still does not open, run:

```bash
xattr -dr com.apple.quarantine /Applications/Ralph4Days.app
open /Applications/Ralph4Days.app
```

8. If there is still a signature/integrity error, run:

```bash
codesign --force --deep --sign - /Applications/Ralph4Days.app
open /Applications/Ralph4Days.app
```

9. Confirm the app launches and send screenshots/error messages back to the maker if anything fails.
