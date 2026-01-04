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

## Configuration

mmtui supports an optional configuration file located at:

`$XDG_CONFIG_HOME/mmtui/mmtui.toml`

Given a relative path, mmtui searches `XDG_CONFIG_HOME` first and then `XDG_CONFIG_DIRS`, using the first existing configuration file found. If no configuration file found, built-in defaults are used. If a configuration file exists, it fully overrides the default configuration. The lookup is based on [freedesktop.org XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/).

The following shows the built-in default configuration used when no configuration file is present:
```toml
fstype_ignore = [
        "tmpfs",
        "ramfs",
        "swap",
        "devtmpfs",
        "devpts",
        "hugetlbfs",
        "mqueue",
        "fuse.portal",
        "fuse.gvfsd-fuse",

]
path_ignore =  [
        "/tmp",
        "/sys",
        "/proc",
]
```

### Options

All options must be present in the configuration file. Options cannot be omitted; use an empty list if no entries are desired.

* `fstype_ignore`
    List of filesystem types to ignore. Any mount whose filesystem type matches an entry in this list will be excluded.

* `path_ignore`
    List of absolute paths to ignore. Any mount point or path matching an entry in this list will be excluded.


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

# release-please
[Release-please is used](https://github.com/googleapis/release-please) in this repository. Because of that there are rules for commit names:

## How should I write my commits?

Release Please assumes you are using [Conventional Commit messages](https://www.conventionalcommits.org/).

The most important prefixes you should have in mind are:

* `fix:` which represents bug fixes, and correlates to a [SemVer](https://semver.org/)
  patch.
* `feat:` which represents a new feature, and correlates to a SemVer minor.
* `feat!:`,  or `fix!:`, `refactor!:`, etc., which represent a breaking change
  (indicated by the `!`) and will result in a SemVer major.
