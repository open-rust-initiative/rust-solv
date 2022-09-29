# rust-solv

## 简介

rust-solv 是一个使用 Rust 实现的基于 SAT 算法的软件包依赖分析库。

## 使用

在使用之前需要先在 `~/.config/rust-solv/config.toml` 编写配置文件，格式形如：

```toml
[repoinfo]
name = "OS"
baseurl = "http://repo.openeuler.org/openEuler-22.03-LTS/OS/$basearch/"
```

之后便可以执行程序，查询在配置文件指定仓库中能否满足指定软件的依赖。

```
$ cargo run package1 package2 ...
```

### How to contribute?

This project enforce the [DCO](https://developercertificate.org).

Contributors sign-off that they adhere to these requirements by adding a Signed-off-by line to commit messages.

```bash
This is my commit message

Signed-off-by: Random J Developer <random@developer.example.org>
```

Git even has a -s command line option to append this automatically to your commit message:

```bash
$ git commit -s -m 'This is my commit message'
```

### License

Freighter is licensed under this Licensed:

* MIT LICENSE ( [LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)

### Acknowledgements
