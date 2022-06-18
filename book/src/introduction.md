# Introduction

Welcome! This book will guide you through the creation and design of a 3D game engine using the [Rust](https://www.rust-lang.org/) programming language. A variety of popular open source libraries will be used to achieve this goal in a reasonable amount of time.

## Purpose

As of June 2022, the resources for learning game engine creation are scarce. The existing resources largely focus on `C++` and cover various rendering techniques rather than topics that would be useful for designing gameplay mechanics. This is excellent! However, the focus for many of these resources is not on building a structured program that goes beyond the scope of tutorial code. This book is meant to be higher level, demonstrating how to build a 3D world and render it in realtime. This will be particularly of use to indie game developers looking to create a 3D game from scratch without getting overwhelmed.

## Why Rust?

Rust is a great alternative to `C++`!  A few of the benefits:

* It provides a smooth workflow for developers with clear, specific error messages from the compiler
* [rustup](https://rustup.rs/) and [cargo](https://github.com/rust-lang/cargo) make managing rust toolchain installations and rust projects straightforward
* The lints from [clippy](https://github.com/rust-lang/rust-clippy) help to improve the code quality and catch certain common mistakes
* [rustfmt](https://github.com/rust-lang/rustfmt) handles formatting the code and code style
* Memory safety. Code written outside of `unsafe` blocks is checked by the [borrow checker](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html?highlight=borrow#references-and-borrowing)

## Target Audience

The target audience of this book is moderately experienced developers with an interest in graphics programming and game development. Prior graphics programming experience will be particularly useful. This is the book I would have wanted to read when first starting out with designing games from scratch.

## What Is Covered

This book is very code-heavy and implementation focused, as opposed to other resources that may be more focused on theory.

## What Is Not Covered

This book will not go into detail on linear algebra concepts or mathematics, as there are already great resources available for deep information those topics. A list of useful external resources for building upon the content of this book can be found in the `Further Reading` section of the appendix.

## Project Repo

All of the source code for the `Phantom` engine built in this book can be found on github:

<https://github.com/matthewjberger/phantom>
