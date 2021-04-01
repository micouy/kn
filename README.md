# `se` - navigate by matching slices of path

`se` is an alternative to `cd`. It matches args against each file's path's components in order. It doesn't track frecency or any other statistics.


# Installation

The project is in it's alpha stage. For now it only works with `fish` shell. To install, follow the instructions:

`git clone https://github.com/micouy/se.git`

`cd se`

Put the binary in a folder that is in `PATH` so that it can be accessed by the script:

`cargo build -Z unstable-options --out-dir DESTINATION --release`

Put this line in your `config.fish`:

`_se init fish | source`

After launching a new fish instance or reloading the config with `source YOUR_FISH_CONFIG_PATH` you'll be able to use the `se` command.


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
$ se tes fo iz
```


# Behavior

* `se` returns shortest paths possible.
  
  `se buzz` will try to return `buzz` rather than `foo/buzz`.
* `se` tries to find each arg in path's components.
  
  `se foo baz` **will not** match `foo`.
* `se` will not try to match more than one arg against any component.
  
  `se th ng` **will not** match `thing` but it **will** match `tthh/i/ii/iii/nngg`.
* `se` will try to match one whole arg against the content of only one component.
  
  `se thingy` **will not** match `thing` or `thi/ngy`.
* `se` matches args in the order provided by the user.
  
  `se foo baz` **will not** match `baz/foo` but it **will** match `foo/baz`.


# Future development

- [x] Use the first arg as a starting directory.
- [x] Use `clap`.
- [x] Enter matched directory.
- [x] Match only directories.
- [ ] Use slashes instead of spaces. This will allow the user to enforce a specific number of components.
- [ ] Add `--help` to `se` function. (How?)
