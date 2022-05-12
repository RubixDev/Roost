# Roost - A Basic Programming Language for Demonstration
*Roost* is a simple example for an interpreted programming language.
Its name is a combination of the language it was written in - [Rust](https://rust-lang.org) - and the word "rooster".

I created this language in conjunction with an obligatory research paper about the structure of a programming language in my 11th grade on school. The resulting paper can be found [here](Facharbeit_Der-Aufbau-einer-Programmiersprache_Silas-Groh_2022-03-07_digital.pdf).
The main focus of the paper are the lexing and parsing steps, regular and context-free languages, and the grammatical definition of a language. After that a short note on compilers and [LLVM](https://llvm.org/) follows and some basic implementation details for a tree-walking interpreter are given.

Most of the paper actually refers to [*rost*](https://github.com/RubixDev/rost), an even more stripped down language acting just as a calculator but following the same principles, because while writing I quickly noticed that even *Roost* is too complex to explain in such a short paper.

<p align="center"><img src="logo.png" alt="Logo of Roost" title="Logo of Roost" width="300"></p>

The logo seen here and on the paper's title page was created by my friend and classmate [Mik Müller](https://github.com/MikMuellerDev).

## See also
- [*rost*](https://github.com/RubixDev/rost): A simple interpreted calculator following the same principles
- [The german research paper](Facharbeit_Der-Aufbau-einer-Programmiersprache_Silas-Groh_2022-03-07_digital.pdf)
- [roost.rubixdev.de](https://roost.rubixdev.de): An online playground for *Roost* using [WebAssembly](https://webassembly.org/)
  - And the corresponding [GitHub repository](https://github.com/RubixDev/roost-web)
- [rost.rubixdev.de](https://rost.rubixdev.de): An online playground for *rost* using [WebAssembly](https://webassembly.org/)
  - And the corresponding [GitHub repository](https://github.com/RubixDev/rost-web)
- [My school's website](http://cfg.wtal.de/)
- [Rust](https://rust-lang.org): The language both *Roost* and *rost* were written in

## Local usage
### 1. Clone this repository
```bash
git clone https://github.com/RubixDev/roost.git && cd roost
```

### 2. Compile the binary
```bash
make release
```

### 3. Copy the binary into your PATH
```bash
sudo cp target/release/roost-cli /usr/local/bin/roost
```
or for just this user:
```bash
cp target/release/roost-cli ~/.local/bin/roost
```

> Note: This step assumes you are running Linux and have your $PATH variable setup correctly. On other operating systems you can either run the binary by specifying the whole path or use `cargo run --release`

### 4. Run *Roost* code
#### Using the REPL/interactive shell
When executing the `roost` command without any extra arguments you enter the REPL for Roost. Here you can simply type expressions and execute them one by one.

#### Running files
To run a file (usually ending with `.ro`) pass the path to that file as the first argument to `roost`. For example:
```bash
roost samples/sample.ro
```
