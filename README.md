# `kn` */n/*

![Github Actions badge](https://github.com/micouy/kn/actions/workflows/build-and-tests.yml/badge.svg)
![crates.io badge](https://img.shields.io/crates/v/kn.svg)

<p align="center">
<img src="assets/demo.svg" width="70%" />
</p>

`kn` is an alternative to `cd`. It lets you navigate quickly by typing abbreviations. It doesn't track frecency or any other statistics - it's **dumb, predictable and good at one thing**.

**WARNING**: This project is in its alpha stage.


# Features

[Demo](https://asciinema.org/a/406626?speed=2)

```
test-dir
├── boo
│  └── buzz
├── buzz
├── bazz
└── foo
   ├── bizz
   └── bazz
```

```
$ kn              # enter ~
$ kn ~            # also enter ~

$ kn -            # go back to previous location

$ kn tes/fo/iz    # enter test-dir/foo/bizz

$ kn tes/baz      # enter test-dir/bazz

$ kn tes/-/baz    # enter test-dir/foo/bazz

$ kn -/bo/uzz     # enter test-dir/boo/buzz
```

`kn ~`, `kn ..`, `kn .`, `kn /` work just like with cd.

`kn a/-/b` means "Go to `b` in any dir which is in `a`.".
`kn -/a` means "Go to `a` in any dir which is in current dir.".
`kn /-/a` means "Go to `a` in any dir which is in root.".

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

## Building the binary

Install `kn` from `crates.io`:

1. `cargo install kn`

**OR**

Build binary from source:

1. `git clone https://github.com/micouy/kn.git`
2. `cd kn`
3. Put the binary in a folder that is in `PATH`:

   `cargo build -Z unstable-options --out-dir DIR_IN_PATH --release`

   Or just build it and copy the binary to that dir:

   `cargo build --release`

   `cp target/release/_kn DIR_IN_PATH`


## Configuring your shell

1. Add this line to the config of your shell (notice the underscore in `_kn`):

   * **fish** (usually `~/.config/fish/config.fish`):

     `_kn init fish | source`
   * **bash** (usually `~/.bashrc`):

     `eval "$(_kn init bash)"`

   * **zsh** (usually `~/.zshrc`):

     `eval "$(_kn init zsh)"`
2. To be able to use `kn`, reload your config or launch a new shell instance.


# Help wanted

In this project I have entered a lot of areas I have little knowledge about. Contributions and criticism are very welcome. Here are some things you can do:

- Check the correctness of scripts in [init/](init/).
- Add scripts and installation instructions for shells other than `fish`, `bash` and `zsh`.
- Check regular expressions used in `Abbr::from_string` in `src/query/abbr.rs` to validate abbreviation's components. Are there other characters or sequences which should be prohibited?


# TODO

## Search engine

- [ ] What to do about `.` and `..` in the middle of path abbreviation? With `..` in paths the results would be too unpredictable. Are there any situations when `..` show up in path? The user probably wouldn't type it but a command line tool could return such path.
- [ ] Return objects containing details about the matches (the sequence of `Congruence`s with details about which chars have been matched). This will be useful in interactive mode.
- [ ] Use inodes instead of traversing the directory structure using `read_dir()`. [Guide.](https://fasterthanli.me/series/reading-files-the-hard-way) Are there inodes on other OSes?
- [ ] Read [Falsehoods programmers believe about paths](https://yakking.branchable.com/posts/falsehoods-programmers-believe-about-file-paths/).


## CLI experience

- [ ] Make `kn` somewhat interactive. Tab could confirm the path `kn` has found so far and the search could begin from that location. That would narrow down the search. (Is that possible with `fish` and other shells?)
- [ ] Add `--help` to `kn` function. (How?)
- [ ] Read about [`broot`](https://github.com/Canop/broot).
- [ ] Enable excluding dirs.
