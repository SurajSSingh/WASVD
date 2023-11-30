# WebAssemby Stack Visual Debugger

***THIS PROJECT IS VERY EARLY IN THE PROCESS, EXPECT HEAVY BREAKING CHANGES AND CHANGES TO DOCUMENTATION.***

This project allows one to visualize WebAssembly instructions and execution in a more visual way. I made this to help out with writing Wasm solutions for the Nibbly November [#12in23 on Exercism](https://exercism.org/challenges/12in23). 

## Building
Requires:
* Node
    * I use PNPM, but NPM should work (Yarn may require a bit more setup)
* Cargo

### Steps:
Go to the project directory

1. Install all dependencies for Node/Svelte-side using (p)npm

```shell
cd ./src
# If you have pnpm
pnpm i
# Or for npm
npm install
```

2. Install all dependencies for Rust-side using cargo

```shell
cd ../src-tauri
cargo build
```

3. Run the app
```shell
pnpm run tauri dev
# Or
npm run tauri dev
```

## License
Licensed under the Mozilla Public License Version 2.0, which can be viewed in the LICENSE file or at <https://www.mozilla.org/en-US/MPL/2.0/>.