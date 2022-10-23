## todo_notes

A command line TODO list program to practice concepts from [The Rust Book I/O project.](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)

It creates a directory `.todonotes` containing a config and TODO list files. It will attempt to discover if the user is in a git repository and create or use the associated list, otherwise it will use the default list.

```
Usage: ./todo_notes [options]

Options:
    -a, --add           Add "list item"
    -l, --list          List all items
    -d, --delete        Delete list item n
    -h, --help          Display usage info
```
