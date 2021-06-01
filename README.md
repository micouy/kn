# `kn` */n/*

![Github Actions badge](https://github.com/micouy/kn/actions/workflows/tests.yml/badge.svg)
[![crates.io badge](https://img.shields.io/crates/v/kn.svg)](https://crates.io/crates/kn)

`kn` is an alternative to `cd`. It lets you navigate quickly by typing abbreviations.

<p align="center">
<img src="assets/demo.svg" />
</p>

```bash
cargo install kn
```

Then follow [the installation instructions](#installation).


# Features

* [Interactive mode](#interactive-mode)
* [Normal mode](#normal-mode)
  * [Abbreviations](#abbreviations)
  * [Wildcards](#wildcards)
  * [Multiple dots](#multiple-dots)

## Interactive mode

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

<kbd>Backspace</kbd> removes whole components until it reaches current dir.

To change dir, press <kbd>Enter</kbd>. Note that `kn` enters currently selected suggestion, not the path displayed in gray. This may change in the future.

## Normal mode

### Abbreviations

You can use `kn` just like you'd use `cd`. The difference is that you can search with abbreviations instead of full dir names. **For example, instead of `foo/bar` you can type `fo/ba`.**

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

### Wildcards

You can also use wildcards `-` to avoid typing a dir name altogether i.e. kn `-/beta` to go to `alpha/beta`. Note that `kn alph-/bet-` will not match `alhpa/beta`. In this case `-` functions as a literal character.

```bash
$ kn -/bar            # Wildcards can be used to skip a dir name altogether (changes dir to ./foo/bar/).
```

### Multiple dots

You can use more than two dots in each component of the prefix:

```bash
$ kn ..               # Go to parent dir (as usual).
$ kn ...              # Go to grandparent dir (same as ../..).
$ kn ....             # Go to grand-grandparent dir (same as ../../..).

$ kn ........         # Type as many dots as you want!
$ kn .../....         # This works as well.

$ kn .../..../abbr    # You can put abbreviations after the prefix.
```

If any of `.`, `..`, `/`, `~`, `-` occurs in the argument before normal components, it will work just like in `cd` and it won't be interpreted as an abbreviation.

<details>
  <summary>See examples</summary>
  The mentioned components work as usual:

  ```
  $ kn .
  $ kn ./abbr
  $ kn ..
  $ kn ../..
  $ kn ../../abbr

  $ kn /
  $ kn /abbr

  $ kn ~
  $ kn ~/abbr

  $ kn -
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

**OR (Arch-based Linux only)**

Install `kn` from [ArchLinux User Repository](https://aur.archlinux.org/packages/kn/) using Your favourite `aur` helper:

```bash
yay -S kn
```

**Note:** remember to configure your shell anyway!

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


# The algorithm

`kn` doesn't track frecency or any other statistics. It searches the disk for paths matching the abbreviation. If it finds many matching paths with the same number of components it orders them in such a way:

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
