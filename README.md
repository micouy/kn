# `kn` */n/*

`kn` is an alternative to `cd`. It matches args against each file's path's components in order. It doesn't track frecency or any other statistics.


# Installation

The project is in it's alpha stage. For now it only works with `fish` shell. To install, follow the instructions:

`git clone https://github.com/micouy/kn.git`

`cd kn`

Put the binary in a folder that is in `PATH` so that it can be accessed by the script:

`cargo build -Z unstable-options --out-dir DESTINATION --release`

Put this line in your `config.fish`:

`_kn init fish | source`

After launching a new `fish` instance or reloading the config with `source YOUR_FISH_CONFIG_PATH` you'll be able to use the `kn` command.


# Example

```
test-dir
├── boo
│  └── buzz
├── buzz
└── foo
   ├── bizz
   └── buzz
```

Jump to `tes[t-dir]/fo[o]/[b]iz[z]`.

```
$ kn tes fo iz
```


# Behavior

* `kn` returns shortest paths possible.
  
  `kn buzz` will try to return `buzz` rather than `foo/buzz`.
* `kn` tries to find each arg in path's components.
  
  `kn foo baz` **will not** match `foo`.
* `kn` will not try to match more than one arg against any component.
  
  `kn th ng` **will not** match `thing` but it **will** match `tthh/i/ii/iii/nngg`.
* `kn` will try to match one whole arg against the content of only one component.
  
  `kn thingy` **will not** match `thing` or `thi/ngy`.
* `kn` matches args in the order provided by the user.
  
  `kn foo baz` **will not** match `baz/foo` but it **will** match `foo/baz`.


# TODO

- [x] Use the first arg as a starting directory.
- [x] Use `clap`.
- [x] Enter matched directory.
- [x] Match only directories.
- [x] Move logic from `kn.fish` to the binary.
- [ ] Use slashes instead of spaces. This will allow the user to enforce a specific number of components.
- [ ] Add `--help` to `se` function. (How?)
- [ ] Compare slices with `String::windows` instead of matching regex? Constructing regex from user's input seems hacky, even if it's validated.
- [ ] Make `kn` somewhat interactive. Tab could confirm the path `kn` has found so far and the search could begin from that location. That would narrow down the search. Is that possible with `fish`?
- [ ] Config:
  * Max space between slices. With space 2 `kn a b` would match `a/x/x/b` but not `a/x/x/x/b`.
  * Strict mode. When the wildcard slices are implemented it will be possible to match the components at exact positions, not at any position.
