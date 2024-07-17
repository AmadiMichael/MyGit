# My Git

To test run

### Create test dir:

```zsh
mkdir test_dir && cd test_dir
```

### Initialize my git in it:

```zsh
../my_git.sh init
```

### Create a file:

```zsh
echo "Hello world!" > sample.txt
```

### Get the hash object of that file

```zsh
../my_git.sh hash-object -w sample.txt
```

`-w` means this would add this file's encoded content to the `.git/objects` folder at path `.git/objects/hash[0..2]/hash[2..20]`

If omited it will only print out the hash object and not write it to `.git/objects`

### View the file with cat-file

```zsh
../my_git.sh cat-file -p [hash outputed from execution above]
```

`-p` prints the original content of the hash, `-s` prints the size and `-t` prints the type (blob, tree or commit)

You can also use normal git in this folder and use git log to get past commit hashes and view info on that using `../my_git.sh cat-file -p ...`

### Write everything in the working directory (Tree (directory) objects)

```zsh
../my_git.sh write-tree
```

### View the contents of a tree object hash

```zsh
../my_git.sh ls-tree [hash]
```
