# `kn` */n/*

![Github Actions badge](https://github.com/micouy/kn/actions/workflows/tests.yml/badge.svg)
[![crates.io badge](https://img.shields.io/crates/v/kn.svg)](https://crates.io/crates/kn)

`kn` is an alternative to `cd`. It lets you navigate quickly by typing abbreviations.

<p align="center">
<img src="assets/banner.svg" />
</p>

```fish
cargo install kn
```

Then follow the [configuration instructions](#configuring-your-shell).


# Features

* [Abbreviations](#abbreviations)
* [Wildcards](#wildcards)
* [Multiple dots](#multiple-dots)
* [`--exclude-old-pwd`](#--exclude-old-pwd)


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

```fish
kn foo/bar          # Use `kn` just like `cd`...
kn fo/ba            # ...or navigate with abbreviations! No asterisks required.
kn pho2021          # Type only the significant parts of the dir name. You can skip the middle part.
```


## Wildcards

You can also use wildcards `-` to avoid typing a dir name altogether i.e. `kn -/ba` to go to `foo/bar`. Note that `kn f-/b-` will not match `foo/bar`. In this case `-` functions as a literal character.

```fish
kn -/bar            # Wildcards can be used to skip a dir name altogether (changes dir to ./foo/bar/).
```


## Multiple dots

`kn` splits the arg into two parts, a prefix and a sequence of abbreviations. The prefix may contain components like `c:/`, `/`, `~/`, `.`, `..` and it is treated as a literal path. It may also contain components with more than two dots, which are interpreted like this:

```fish
kn ..               # Go to parent dir (as usual).
kn ...              # Go to grandparent dir (same as ../..).
kn ....             # Go to grand-grandparent dir (same as ../../..).

kn ........         # Type as many dots as you want!
kn .../....         # This works as well.

kn .../..../abbr    # You can put abbreviations after the prefix.
```

**If any of the mentioned components occurs in the path after an abbreviation, it is treated as an abbreviation.**

```fish
kn ./../foo/bar/../baz
#  ^---^                 prefix
#       ^------------^   abbreviations
```

`.` and the first `..` mean *current dir* and *parent dir*, while the second `..` is treated as an abbreviation, that is, it will match a dir name containing two dots.


## `--exclude-old-pwd`

This flag excludes your previous location from the search. You don't have to type it when using `kn`, just set it in your shell script (notice the underscore in `_kn`):

```fish
_kn init --shell fish --exclude-old-pwd
```

It's useful when two paths match your abbreviation and you enter the wrong one:

```fish
my-files/
$ kn d

my-files/dir-1/
$ kn -

my-files/
$ kn d # just press arrow up twice

my-files/dir-2/
$ # success!
```

In order for `kn` to exclude the previous location there must be at least one other match and the provided arg must **not** be a literal path (that is, it must be an abbreviation).


# Installation

Make sure to [configure your shell](#configuring-your-shell) after the installation.


## From `crates.io`

```fish
cargo install kn
```


## From source

1. `git clone https://github.com/micouy/kn.git`
2. `cd kn`
3. Put the binary in a folder that is in `PATH`:

   `cargo build -Z unstable-options --out-dir DIR_IN_PATH --release`

   Or just build it and copy the binary to that dir:

   `cargo build --release`

   `cp target/release/_kn DIR_IN_PATH`


## From the release page

Download a binary of the [latest release](https://github.com/micouy/kn/releases/latest) for your OS and move it to a directory which is in your `$PATH`. You may need to change the binary's permissions by running `chmod +x _kn`.

If there are any problems with the pre-compiled binaries, file an issue.


## Configuring your shell

Then add this line to the config of your shell (notice the underscore in `_kn`):

* **fish** (usually `~/.config/fish/config.fish`):

  `_kn init --shell fish | source`
* **bash** (usually `~/.bashrc`):

  `eval "$(_kn init --shell bash)"`

* **zsh** (usually `~/.zshrc`):

  `eval "$(_kn init --shell zsh)"`

You may also want to enable [the `--exclude-old-pwd` flag](#--exclude-old-pwd). To be able to use `kn`, reload your config or launch a new shell instance.


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
   - `Subsequence(coefficient)` if the abbreviation's component is a subsequence of the path's component. The `coefficient` is the [*Powierża coefficient*](https://github.com/micouy/powierza-coefficient) of these strings.
   Retain only these paths in which all of the components match.
2. Order the paths in reverse lexicographical order (compare the results from right to left). `Complete` then `Prefix` then `Subsequence`. Order paths with `Subsequence` result in ascending order of their `coefficient`'s.
3. Order paths with the same results with [`alphanumeric_sort::compare_os_str`](https://docs.rs/alphanumeric-sort/1.4.3/alphanumeric_sort/fn.compare_os_str.html).
