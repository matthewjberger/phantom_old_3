# Getting Started

## Dependencies

This project requires the [Rust programming language](https://www.rust-lang.org/).

The easiest way to get access to the Vulkan dependencies is to install [the Vulkan SDK](https://vulkan.lunarg.com/sdk/home). This provides access to the Vulkan Configurator, glslangvalidator, the debug layers, and other useful tools for working with Vulkan.

The latest version of `GCC` and `CMake` will also be required for ffi bindings to C/C++ libraries.

## Development Environment Setup

Using [vscode](https://code.visualstudio.com/) and [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer) with the rust-analyzer [vscode extension](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) is recommended. However, any Rust development environment you are comfortable with will work.

The [official Rust book](https://doc.rust-lang.org/book/) is a great resource if you are new to Rust.

### Quick Windows Setup

On windows, installing programs can be trickier than on other platforms. It is recommended to use a package manager such as [Scoop](https://scoop.sh/) or [chocolatey](https://chocolatey.org/).

First, make sure [PowerShell 5](https://aka.ms/wmf5download) (or later, include [PowerShell Core](https://docs.microsoft.com/en-us/powershell/scripting/install/installing-powershell-core-on-windows?view=powershell-6)) and .[NET Framework 4.5](https://www.microsoft.com/net/download) (or later) are installed. Then run:

```powershell
# Install scoop
Set-ExecutionPolicy RemoteSigned -scope CurrentUser
Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')

# Scoop uses git to update itself and 7zip to extract archives
scoop install git 7zip 

# Install the project's dependencies
scoop install gcc cmake rustup

# Set the stable rust toolchain as the default toolchain
rustup default stable

# Install vscode, kept in a separate bucket called the 'extras' bucket
scoop bucket add extras
scoop install vscode
```
