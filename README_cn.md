# Distro pioneer

每次使用新的发行版，都需要将以前写的脚本、别名、环境变量、安装的软件全部重新安装一遍，太麻烦了，所以做了这个工具。

做这个工具一是为了方便我在WSL，服务器，编译主机，以及新安装的环境上，快速统一我的一系列命令、别名、脚本，简化开荒的步骤；二是我比较喜欢rust，想锻炼rust的熟练度。

这个工具的目标是：一行命令将新的系统环境开荒完成，比如`sh install.sh`，再简单一点，将文件放到服务器上，通过网络下载并运行`curl https:://file.server.com/install.sh | sh`。

所以这是一个基于debian系列发行版的，系统开荒工具


## binary说明

- pioneer: 用于distro开荒的主题程序，所有安装软件、别名等操作都由此实现
- script_builder: 用于生成一键安装脚本，这个软件会将pioneer和所有配置文件全部打包到一个shell脚本内


## 尝试一下


构建installer.sh脚本

```sh
cargo r --bin script_builder -- -p pioneer -c test_example/*.toml
```

在容器里测试

```sh
docker run --rm \
-itv $(realpath .):/ws \
-w /ws ubuntu
```

```sh
bash installer.sh
```
