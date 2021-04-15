# `kn` */n/*

![Github Actions badge](https://github.com/micouy/kn/actions/workflows/tests.yml/badge.svg)
[![crates.io badge](https://img.shields.io/crates/v/kn.svg)](https://crates.io/crates/kn)

<p align="center">
<img src="assets/demo.svg" />
</p>

`kn` is an alternative to `cd`. It lets you navigate quickly by typing abbreviations. It doesn't track frecency or any other statistics.

**WARNING**: This project is in its alpha stage.


# Features

<p align="center">
<a href="https://asciinema.org/a/406626?speed=2"><img src="https://asciinema.org/a/406626.svg" alt="Demo" /></a>
</p>

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
$ kn fo/br            # ...or navigate with abbreviations!
$ kn pho2021          # Type only the significant parts of the dir name. You can skip the middle part.
$ kn -/bar            # Wildcards to skip a dir name altogether (./foo/bar/).
```

The basic feature of `kn` is that it lets you type abbreviations instead of full dir names. Instead of `foo/bar` you can type `fo/ba`.

You can also use wildcards `-` to avoid typing a dir name altogether i.e. kn `-/beta` to go to `alpha/beta`. **Note that `kn alph-/bet-` will not match `alhpa/beta`. In this case `-` functions as a literal character.**

These also work:

```bash
$ kn .           # Stay in current dir.
$ kn ..          # Enter parent dir.
$ kn /           # Enter root dir.
$ kn             # Enter home dir.
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
- Review regular expressions used in `Abbr::from_string` in `src/query/abbr.rs` to validate abbreviation's components. Are there other characters or sequences which should be prohibited?
- Review Github Actions workflows in [.github/workflows/](.github/workflows/).


# TODO

## Interactive mode

- [ ] Display 3 suggestions at most, preferably right below the prompt.
- [ ] Handle each component separately, not like `fzf` which matches whole paths.
- [ ] Make backspace remove whole component, like in `amp`.


## Search engine

- [ ] What to do about `.` and `..` in the middle of path abbreviation? With `..` in paths the results would be too unpredictable. Are there any situations when `..` show up in path? The user probably wouldn't type it but a command line tool could return such path.
- [ ] Return objects containing details about the matches (the sequence of `Congruence`s with details about which chars have been matched). This will be useful in interactive mode.
- [ ] Use inodes instead of traversing the directory structure using `read_dir()`. [Guide.](https://fasterthanli.me/series/reading-files-the-hard-way) Are there inodes on other OSes?
- [ ] Read [Falsehoods programmers believe about paths](https://yakking.branchable.com/posts/falsehoods-programmers-believe-about-file-paths/).


## CLI experience

- [ ] Add `--help` to `kn` function.
- [ ] Enable excluding dirs.
