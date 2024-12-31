# mmtui

TUI disk mount manager for TUI file managers for Linux

![](screencast.gif)

# Dependencies

- udisks2

# Binaries

Latest prebuilt static binary you can find here: https://github.com/SL-RU/mmtui/releases

# Build

```
cargo build --release
```

Binary will be in `target/release/mmtui`

## AUR

On Archlinux you can install from AUR, package is named `mmtui-bin`: https://aur.archlinux.org/packages/mmtui-bin

# Integrations

## Yazi - mount.yazi

Plugin for yazi file manager: https://github.com/SL-RU/mount.yazi

## Ranger file manager

https://github.com/SL-RU/ranger_udisk_menu is deprecated, now you can use this application instead. Download or build `mmtui` and add it location to $PATH environment variable or substitute location in the variable MMTUI_PATH in the script below.

Add this to `ranger config/commands.py`:

```Python
class mount(Command):
    """:mount.

    Show menu to mount and unmount.
    """

    MMTUI_PATH = "mmtui"

    def execute(self):
        """Show menu to mount and unmount."""
        import os
        import tempfile
        (f, p) = tempfile.mkstemp()
        os.close(f)
        self.fm.execute_console(
            f'shell bash -c "{self.MMTUI_PATH} 1> {p}"'
        )
        with open(p, 'r') as f:
            d = f.readline().strip()
            if os.path.exists(d):
                self.fm.cd(d)
        os.remove(p)
```
