# Rust (mini)-shell
![rust](https://img.shields.io/badge/Rust-444444?logo=rust&logoColor=red)
![Last Commit](https://img.shields.io/github/last-commit/spaghetoz/rust-shell)

A basic Unix shell implemented in Rust, created as a learning project to explore Rust’s code organisation and memory safety compared to C.

<br>

```shell
# Shell behavior demonstration: redirections, pipes and logical operators

# Basic navigation
$ /home> cd folder
$ /home/folder> ls 
text_file.txt

# Command redirection (append)
$ /home/folder> ls -l -a >> text_file.txt

# Pipe and redirection combination
$ /home/folder> cat text_file.txt | wc -l > file.txt

# Sequential execution with ;
$ /home/folder> echo hello ; inexistant_program ; echo hello2
hello
Command execution error: No such file or directory
hello2

# Conditionnal execution with &&
$ /home/folder> echo hello && inexistant_program && echo hello2
hello
Command execution error: No such file or directory

# Conditionnal execution with ||
$ /home/folder> inexistant_program || echo hello || echo hello2
Command execution error: No such file or directory
hello

# Pipes chaining
$ /home/folder> cat text_file.txt | wc -l | head
4
$ /home/folder> exit
```


### Supported platforms: 
- Linux (tested on Ubuntu WSL)
- Maybe MacOS

### Features
- Basic commands lexing and parsing
- Commands execution : 
    - Simple commands (for example `ls -l /`)
    - Redirections (<, >>, >>, 2>)
    - Pipes commands
- Commands chaining (; && ||)
- Pipes chaining
- Enriched line editing and history thanks to the [Rusty lines](https://github.com/kkawakam/rustyline) library
- Simple background command with & (example: sleep 2 ; ls > test.txt &)

### How to use : `cargo run` 

### Third party libraries

- `Logos` for lexing
- `Nix` for advanced commands execution

### Next features 

- [ ] Jobs management  (fd, bg, jobs)
- [ ] Signal handling for running commands
- [ ] Environment variables  (echo, $PATH, $?, $VAR etc..)
- [ ] Prompt string with git infos
- [ ] Command auto-completion and suggestion
