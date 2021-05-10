# `kn` */n/*

![Github Actions badge](https://github.com/micouy/kn/actions/workflows/tests.yml/badge.svg)
[![crates.io badge](https://img.shields.io/crates/v/kn.svg)](https://crates.io/crates/kn)

<p align="center">
<img src="assets/demo.svg" />
</p>

`kn` is an alternative to `cd`. It lets you navigate quickly by typing abbreviations. It doesn't track frecency or any other statistics.

**WARNING**: This project is in its alpha stage.


# Features

## *Interactive mode!*

**Enter interactive mode**
```bash
$ kn
```

**Filter suggestions by typing**

![demo](assets/filter.gif)

**Select with <kbd>Tab</kbd> and <kbd>Shift</kbd> + <kbd>Tab</kbd>**

You can also use <kbd>Ctrl</kbd> + <kbd>hjkl</kbd>.

![demo](assets/select.gif)

**Enter dir with <kbd>/</kbd>**

![demo](assets/enter-dir.gif)

<kbd>Backspace</kbd> removes the whole input. If the input is empty, it enters the parent dir (unless the search started at current dir).

To change dir, press <kbd>Enter</kbd>. Note that `kn` enters currently selected dir, not the path displayed in gray. This may change in the future.


## Normal mode

You can use `kn` just like `cd`. The difference is that you can search with abbreviations instead of full dir names. For example, instead of `foo/bar` you can type `fo/ba`.

```
.
├── foo
│  └── bar
├── bar
├── photosxxxxxxxxxxx2021
└── photosxxxxxxxxxxx2020
```

```bash
$ kn foo/bar          # Use `kn` just like `cd`...
$ kn fo/br            # ...or navigate with abbreviations! No asterisks required.
$ kn pho2021          # Type only the significant parts of the dir name. You can skip the middle part.
```

You can also use wildcards `-` to avoid typing a dir name altogether i.e. kn `-/beta` to go to `alpha/beta`. **Note that `kn alph-/bet-` will not match `alhpa/beta`. In this case `-` functions as a literal character.**

```bash
$ kn -/bar            # Wildcards is used to skip a dir name altogether (changes dir to ./foo/bar/).
```

These work as usual:

```bash
$ kn .           # Stay in current dir.
$ kn ..          # Enter parent dir.
$ kn /           # Enter root dir.
$ kn ~           # Also enter home dir.
$ kn -           # Go to previous location.
```

<details>
<summary>Details about the ordering of found paths</summary>
If `kn` finds many matching paths with the same number of components it orders them in such a way:

1. Complete matches before partial matches. All matches by wildcards are equal. There can't be a wildcard and a complete/partial match at the same depth.
2. Partial matches with smaller Levenshtein distance first.
3. The first component (the component at the smallest depth) is the most significant and so on.

Running `kn a/-/b` on paths below returns them in the following order:

```
apple/x/b      Partial(4) / Wildcard / Complete      1.
               =            =          !=
apple/y/bee    Partial(4) / Wildcard / Partial(_)    2.
```

```
apple/x/bo     Partial(4) / Wildcard / Partial(1)    1.
               =            =          !=
apple/y/bee    Partial(4) / Wildcard / Partial(2)    2.
```

```
a/x/bo         Complete   / Wildcard / Partial(1)    1.
               !=           -          -
apple/y/b      Partial(4) / Wildcard / Complete      2.
```
</details>


# Installation

## Getting the binary

Install `kn` from `crates.io`

```bash
cargo install kn
```

**OR**

<details>
<summary>Build binary from source</summary>

1. `git clone https://github.com/micouy/kn.git`
2. `cd kn`
3. Put the binary in a folder that is in `PATH`:

   `cargo build -Z unstable-options --out-dir DIR_IN_PATH --release`

   Or just build it and copy the binary to that dir:

   `cargo build --release`

   `cp target/release/_kn DIR_IN_PATH`
</details>

**OR**

Download a binary of the [latest release](https://github.com/micouy/kn/releases/latest) for your OS and move it to a directory which is in your `$PATH`. You may need to change the binary's permissions by running `chmod +x _kn`.

If there are any problems with the pre-compiled binaries, file an issue.


## Configuring your shell

Then add this line to the config of your shell (notice the underscore in `_kn`):

* **fish** (usually `~/.config/fish/config.fish`):

  `_kn init fish | source`
* **bash** (usually `~/.bashrc`):

  `eval "$(_kn init bash)"`

* **zsh** (usually `~/.zshrc`):

  `eval "$(_kn init zsh)"`

To be able to use `kn`, reload your config or launch a new shell instance.


# Help wanted

In this project I have entered a lot of areas I have little knowledge about. Contributions and criticism are very welcome. Here are some things you can do:

- Check the correctness of scripts in [init/](init/).
- Add scripts and installation instructions for shells other than `fish`, `bash` and `zsh`.
- Review Github Actions workflows in [.github/workflows/](.github/workflows/).


# TODO

## Interactive mode

- [ ] When displaying location, trim it down to the last n components.
- [ ] Handle `..`, `/`, `.` and `~` in the prefix. Expand `~` to `$HOME`? [Difference between `$HOME` and tilde.](https://stackoverflow.com/questions/11587343/difference-between-home-and-tilde)


## Search engine

- [ ] What to do about `.` and `..` components in the middle of path abbreviation? With `..` in paths the results would be too unpredictable. Are there any situations when `..` would show up in path? The user probably wouldn't type it but a command line tool could return such path.


## CLI experience

- [ ] Add `--help` to `kn` function.
- [ ] Enable excluding dirs.
