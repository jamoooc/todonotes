## todo_notes

A command line TODO list program to practice concepts from [The Rust Book I/O project.](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)

It creates a directory `.todo_notes` containing a config and TODO list files. It will attempt to discover if the user is in a git repository and create or use the associated list, otherwise it will use the default list.

```
Usage: ./todo_notes [options]

Options:
    -a, --add           Add "list item"
    -l, --list          List all items
    -d, --delete        Delete list item n
    -h, --help          Display usage info
```

```
$ ./todo_notes -a "first item"`

Creating directory: "/Users/jamoooc/.todo_notes"
Creating task file: "/Users/jamoooc/.todo_notes/default.txt"
Found git repository. Using todo list: todo_notes
Creating task file: "/Users/jamoooc/.todo_notes/todo_notes.txt"
Added new item: 01. first item
01. first item
```

```
$ ./todo_notes -a "second item"

Found git repository. Using todo list: todo_notes
Added new item: 02. second item
01. first item
02. second item
```

```
$ ./todo_notes -l

Found git repository. Using todo list: todo_notes
01. first item
02. second item
```

```
$ ./todo_notes -d 1

Found git repository. Using todo list: todo_notes
Deleted list item: 1
01. second item
```
