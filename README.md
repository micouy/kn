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

Then follow the [configuration instructions](#configuring-your-shell).


# Features

* [Abbreviations](#abbreviations)
* [Wildcards](#wildcards)
* [Multiple dots](#multiple-dots)

## Abbreviations

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

## Wildcards

You can also use wildcards `-` to avoid typing a dir name altogether i.e. kn `-/beta` to go to `alpha/beta`. Note that `kn alph-/bet-` will not match `alhpa/beta`. In this case `-` functions as a literal character.

```bash
$ kn -/bar            # Wildcards can be used to skip a dir name altogether (changes dir to ./foo/bar/).
```

## Multiple dots

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

`kn` doesn't track frecency or any other statistics. It searches the disk for paths matching the abbreviation. If it finds multiple matching paths, it orders them in such a way:

1. Compare each component against the corresponding component of the abbreviation. The components of the path may or may not match the abbreviation. If a component matches the abbreviation, there are three possible results:
   - `Complete` if the corresponding components are equal.
   - `Prefix` if the abbreviation's component is a prefix of the path's component.
   - `Subsequence(coefficient)` if the abbreviation's component is a subsequence of the path's component. The `coefficient` is the *Powierża coefficient* of these strings.
   Retain only these paths in which all of the components match.
2. Order the paths in reverse lexicographical order (compare the results from right to left). `Complete` then `Prefix` then `Subsequence`. Order paths with `Subsequence` result in ascending order of their `coefficient`'s.
3. Order paths with the same results with [`alphanumeric_sort::compare_os_str`](https://docs.rs/alphanumeric-sort/1.4.3/alphanumeric_sort/fn.compare_os_str.html).
