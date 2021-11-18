
# esp-create-project

esp-create-project is a utility that simplifies the creation of a ESP32 IDF
project. You only need to invoke the utility and specify a few options!

### Note
You need internet connection to use this CLI because it downloads a template

## Features

- Create a new project
- Specify the programming language to use in the project (C or C++)
- Initialize a project as a git repo
- Specify C++ standard version (11, 14 and 17)
- Cross platform
- Written in Rust

## Usage

To create a new project just invoke the utility

`esp-create-project [name/folder]`, the default name is `esp-new-project`

After invoking CLI, it'll prompt you about the options of the project, which comes in the following order:

* Programming language (default is C)
* Initialize a git repo? (you need git to create it)

## Roadmap
[Roadmap](https://github.com/Alan5142/esp-create-project/wiki/Roadmap)

## License

[MIT](https://choosealicense.com/licenses/mit/)

  
