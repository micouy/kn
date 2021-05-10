# Changelog

The format is inspired by [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## `0.2.0` - 2021-05-10

### Add
- Add changelog.
- Add interactive mode.
  - Add navigation with <kbd>Tab</kbd> and <kbd>Shift</kbd> + <kbd>Tab</kbd> or <kbd>Ctrl</kbd> + <kbd>hjkl</kbd> .
- Add demos in [`README.md`](README.md).


### Change
- Change shell scripts so that calling `kn` without args will enter interactive mode instead of changing current dir to `~`.
- Move search to its own module.


## `0.1.0` - 2021-04-12

### Add
- Add normal mode.
  - Handle abbreviations.
  - Handle prefix (`/`, `~`, `.`, etc.).
  - Handle wildcards (`-`).
- Add shell scripts for `bash`, `fish` and `zsh`.
- Add [`LICENSE.txt`](LICENSE.txt).
- Add GitHub workflows.
