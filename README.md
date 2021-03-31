# `se` - cd by matching parts of path

`se` is an alternative to `cd`. It matches args against each file's path's components in order. It doesn't track frecency or any other statistics.

For now it only prints the generated regular expressions and found files/dirs.


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

Jump to `te[st-dir]/fo[o]/[b]iz[z]`.

```
$ se te fo iz
[^.*te.*$, ^.*fo.*$, ^.*iz.*$]
["./test-dir/foo/bizz"]
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

* Enter matched directory.
* Match only directories.
* Wildcard args (`.`) to enforce a specific number of components.
* Use the first arg as a starting directory.
