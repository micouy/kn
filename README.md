# `kn` */n/*

`kn` is an alternative to `cd`. It traverses the file tree and  matches args against each file's path's components. It doesn't track frecency or any other statistics.

**WARNING**: This project is in it's alpha stage.


# Installation

1. `git clone https://github.com/micouy/kn.git`
2. `cd kn`
3. Put the binary in a folder that is in `PATH` so that it can be accessed by the script:

   `cargo build -Z unstable-options --out-dir DESTINATION --release`
4. Change the config of your shell:

   * `fish` (usually `~/.config/fish/config.fish`):

     `_kn init fish | source`
   * `bash` (usually `~/.bashrc`):

     `eval "$(_kn init bash)"`

   * `zsh` (usually `~/.zsh`):

     `eval "$(_kn init zsh)"`
5. You can set options by passing args to `_kn init` in your config file. For example in `fish` you do it like this:

   `_kn init fish --first-max-depth 3 --next-max-depth 3 | source`

   Read more about options below.
6. After reloading the config or launching a new shell instance you'll be able to use `kn`.


# Example

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
$ kn                # enter ~

$ kn -              # go back to previous location

$ kn tes/fo/iz      # enter test-dir/foo/bizz

$ kn tes biz        # enter test-dir/foo/bizz

$ kn buz            # enter test-dir/buzz

$ kn tes/-/baz      # enter test-dir/foo/bazz
```

`kn ..`, `kn .`, `kn /` also work.


# Behavior

* `kn` checks if the first arg is a valid path. If it is, the search begins there.
* Loose slices: `kn a b` means "Match `b` after `a` (at depth 0 or deeper).".
* Glued slices: `kn a/b` means "Match `b` right after `a` (at depth 0).".
* Wildcards:

  `kn a/-/b` means "Match `b` at depth 1 below `a`.".

  `kn a - b` means "Match `b` at depth 1 below `a` or deeper.".
* `kn` matches at most one slice against each component of the path (meaning `abba` matches `a`, not `a` AND `b`).


# Options

* `--first-max-depth` sets the max depth at which the first match must occur. If set to 0, the first match must occur directly in the start dir. This works well if you want `kn` to work just like `cd` but with shortcuts for dir names.
* `--next-max-depth` sets the max relative depth at which each successive match must occur. If set to 0, each match must occur directly inside the previously matched dir.


# Help wanted

In this project I have entered a lot of areas I have little knowledge about. Contributions and criticism are very welcome. Here are some small things you can do to help me:

- Check the correctness of scripts in [init/](init/).
- Add scripts and installation instructions for shells other than `fish`, `bash` and `zsh`.


# TODO

## Patterns/Slices

- [ ] Remove loose slices?

  Rationale: This would allow `kn` to be good at one thing. It would be just like `cd`, just easier to use. It could guess dir names instead of matching literals and the search times would be much shorter. It would be much more predictable too and would play nicely with interactive mode.
- [x] Use slashes to enforce "glued" slices (slices that must be matched one right after the other). Use spaces to allow for "loose" slices (slices that can be matched a number of components apart from each other).
  - [x] Add `PathSlice::Glued` and `PathSlice::Loose`. ~For now `kn` only uses `PathSlice::Loose`.~
  - [x] Parse args properly.
- [x] Compare slices with `str::match` instead of matching regex? Constructing regex from user's input seems hacky, even if it's validated.
- [x] Use `-` as a wildcard pattern. (Any alternatives to "-"?)
  - `a b` would mean `b` after `a`.
  - `a/b` would mean `b` **right after** `a`.
  - `a/-/b` would mean `b` exactly one dir after from `a`.
  - `a - b` would mean `b` at least one dir after `a`.
  - `.` can be used in the beginning. `./a` would mean `a` in current dir and `/a` would mean `a` in `/`.
- [x] Try to interpret the longest prefix of the first sequence as a literal path.
- [ ] Ignore partial matches if there's a complete match in this subdir?
- [ ] Make `abc` match `axxxbxxxc`? This would allow the user to only type the crucial parts of the path.


## CLI experience

- [x] Enter matched directory.
- [x] Use the first arg as a starting directory.
- [x] Use `clap`.
- [x] Configure `kn` in `config.fish` by passing flags to `kn init`.
  - [x] `--max-space` - Max space between slices. With space 2 `kn a b` would match `a/x/x/b` but not `a/x/x/x/b`.
  - [x] `--max-distance` - Max distance from start dir to the first match. Right now `kn` continues to search paths even if they do not match the slices in case their children match.
  - [ ] Fail quietly on invalid args? How to configure `clap`?
  - [x] Respect options in search.
- [ ] Make `kn` somewhat interactive. Tab could confirm the path `kn` has found so far and the search could begin from that location. That would narrow down the search. (Is that possible with `fish` and other shells?)
- [ ] Add `--help` to `kn` function. (How?)
- [ ] Return all matched results at the same depth (maybe order them in some way) and make the shell script decide which one to use.
- [x] Add support for other shells.
- [ ] Read about [`broot`](https://github.com/Canop/broot).


## Search engine

- [x] Match only directories.
- [x] Make the search engine generic and add tests with a mock file system/search engine.
- [ ] Use inodes instead of traversing the directory structure using `read_dir()`. [Guide.](https://fasterthanli.me/series/reading-files-the-hard-way) Are there inodes on other OSes?


## Other

- [ ] Think of a better terminology/analogy. "Slice of path" is already used in [`Path`'s docs](https://doc.rust-lang.org/std/path/struct.Path.html). "Subseries"?
- [x] Add debug info.
