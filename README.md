# 成功编译了。Android 安卓32位armv7 和。 64位arm64版本的微软的edit。终于可以在Termux 上跑了。但是但是有缺陷，因为安卓系统缺少ICU库， International Components for Unicode（Unicode 国际组件）所以搜索相关功能无法使用。又但是，用Rust原生的正则库写了搜索替换功能，删除了对ICU库的引用，现在可以独立实现搜索功能了，但是缺点是体积膨胀了5倍多，但是也是很迷你的，（搜索下一个可以用F3功能键，完美复刻以前dos版edit.com的功能)，适用Termux。

普通搜索和正则Regex搜索的不同：
   1. 不点钩（默认模式 / 普通模式）：
       * 所见即所得。您输入什么，它就找什么。
       * 例如：您输入 .，它就只找“点号”这个字符。
       * 例如：您输入 \d，它就找“反斜杠和d”这两个字符。
       * 底层原理：虽然我在底层都用了正则引擎，但如果不点这个钩，我会自动把您输入的特殊字符（如 . * ? 等）加上转义符，强制让它们变成普通字符。

   2. 点上钩（正则模式）：
       * 使用正则表达式语法。您输入的字符会被当作“指令”来解析。
       * 例如：您输入 .，它会匹配任意单个字符（不仅是点号，也包括 a, b, 1, 2 等）。
       * 例如：您输入 \d，它会匹配任意数字。
       * 例如：您输入 ^Hello，它会匹配行首的 Hello。

  总结：
   * 如果您只是想找一段普通的文本（比如找个单词、找个标点），不要点钩。
   * 如果您需要高级匹配（比如“找所有以A开头的单词”、“找所有邮箱地址”），才需要点钩。
 关于正则表达式如何使用，请研究https://www.runoob.com/regexp/regexp-intro.html 或者问AI


# ![Application Icon for Edit](./assets/edit.svg) Edit

A simple editor for simple needs.

This editor pays homage to the classic [MS-DOS Editor](https://en.wikipedia.org/wiki/MS-DOS_Editor), but with a modern interface and input controls similar to VS Code. The goal is to provide an accessible editor that even users largely unfamiliar with terminals can easily use.

![Screenshot of Edit with the About dialog in the foreground](./assets/edit_hero_image.png)

## Installation

[![Packaging status](https://repology.org/badge/vertical-allrepos/microsoft-edit.svg?exclude_unsupported=1)](https://repology.org/project/microsoft-edit/versions)

You can also download binaries from [our Releases page](https://github.com/microsoft/edit/releases/latest).

### Windows

You can install the latest version with WinGet:
```powershell
winget install Microsoft.Edit
```

## Build Instructions

* [Install Rust](https://www.rust-lang.org/tools/install)
* Install the nightly toolchain: `rustup install nightly`
  * Alternatively, set the environment variable `RUSTC_BOOTSTRAP=1`
* Clone the repository
* For a release build, run:
  * Rust 1.90 or earlier: `cargo build --config .cargo/release.toml --release`
  * otherwise: `cargo build --config .cargo/release-nightly.toml --release`

### Build Configuration

During compilation you can set various environment variables to configure the build. The following table lists the available configuration options:

Environment variable | Description
--- | ---
`EDIT_CFG_ICU*` | See [ICU library name (SONAME)](#icu-library-name-soname) for details.
`EDIT_CFG_LANGUAGES` | A comma-separated list of languages to include in the build. See [i18n/edit.toml](i18n/edit.toml) for available languages.

## Notes to Package Maintainers

### Package Naming

The canonical executable name is "edit" and the alternative name is "msedit".
We're aware of the potential conflict of "edit" with existing commands and recommend alternatively naming packages and executables "msedit".
Names such as "ms-edit" should be avoided.
Assigning an "edit" alias is recommended, if possible.

### ICU library name (SONAME)

This project _optionally_ depends on the ICU library for its Search and Replace functionality.
By default, the project will look for a SONAME without version suffix:
* Windows: `icuuc.dll`
* macOS: `libicuuc.dylib`
* UNIX, and other OS: `libicuuc.so`

If your installation uses a different SONAME, please set the following environment variable at build time:
* `EDIT_CFG_ICUUC_SONAME`:
  For instance, `libicuuc.so.76`.
* `EDIT_CFG_ICUI18N_SONAME`:
  For instance, `libicui18n.so.76`.

Additionally, this project assumes that the ICU exports are exported without `_` prefix and without version suffix, such as `u_errorName`.
If your installation uses versioned exports, please set:
* `EDIT_CFG_ICU_CPP_EXPORTS`:
  If set to `true`, it'll look for C++ symbols such as `_u_errorName`.
  Enabled by default on macOS.
* `EDIT_CFG_ICU_RENAMING_VERSION`:
  If set to a version number, such as `76`, it'll look for symbols such as `u_errorName_76`.

Finally, you can set the following environment variables:
* `EDIT_CFG_ICU_RENAMING_AUTO_DETECT`:
  If set to `true`, the executable will try to detect the `EDIT_CFG_ICU_RENAMING_VERSION` value at runtime.
  The way it does this is not officially supported by ICU and as such is not recommended to be relied upon.
  Enabled by default on UNIX (excluding macOS) if no other options are set.

To test your settings, run `cargo test` again but with the `--ignored` flag. For instance:
```sh
cargo test -- --ignored
```
