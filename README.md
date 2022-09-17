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