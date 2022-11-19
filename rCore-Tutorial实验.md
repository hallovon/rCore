# rCore-Tutorial实验

## 实验环境配置

1. **windows配置wsl2环境**

   ```shell
   # 启用 Windows 功能：“适用于 Linux 的 Windows 子系统”
   >> dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart

   # 启用 Windows 功能：“已安装的系统虚拟机平台”
   >> dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart

   # <Distro> 改为对应从微软应用商店安装的 Linux 版本名，比如：`wsl --set-version Ubuntu 2`
   # 如果你没有提前从微软应用商店安装任何 Linux 版本，请跳过此步骤
   >> wsl --set-version <Distro> 2

   # 设置默认为 WSL 2，如果 Windows 版本不够，这条命令会出错
   >> wsl --set-default-version 2
   ```

2. **配置C开发环境**

   ```shell
   $ sudo apt-get update && sudo apt-get upgrade
   $ sudo apt-get install git build-essential gdb-multiarch qemu-system-misc gcc-riscv64-linux-gnu binutils-riscv64-linux-gnu
   ```

3. **配置Rust开发环境**

   ```shell
   curl https://sh.rustup.rs -sSf | sh
   ```

   修改crate源,  **~/.cargo** 目录下新建 **config** 文件，写入：

   ```shell
   [source.crates-io]
   registry = "https://github.com/rust-lang/crates.io-index"
   replace-with = 'ustc'
   [source.ustc]
   registry = "git://mirrors.ustc.edu.cn/crates.io-index"
   ```

   安装并修改rust为nightly版：

   ```shell
   rustup install nightly
   rustup default nightly
   ```

   安装rust相关软件包

   ```shell
   rustup target add riscv64gc-unknown-none-elf
   cargo install cargo-binutils --vers =0.3.3
   rustup component add llvm-tools-preview
   rustup component add rust-src
   ```

4. **QEMU模拟器安装**

   ```shell
   # 安装编译所需的依赖包
   sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
                 gawk build-essential bison flex texinfo gperf libtool patchutils bc \
                 zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev libsdl2-dev \
                 git tmux python3 python3-pip ninja-build
   # 下载源码包
   # 如果下载速度过慢可以使用我们提供的百度网盘链接：https://pan.baidu.com/s/1dykndFzY73nqkPL2QXs32Q
   # 提取码：jimc
   wget https://download.qemu.org/qemu-7.0.0.tar.xz
   # 解压
   tar xvJf qemu-7.0.0.tar.xz
   # 编译安装并配置 RISC-V 支持
   cd qemu-7.0.0
   ./configure --target-list=riscv64-softmmu,riscv64-linux-user  # 如果要支持图形界面，可添加 " --enable-sdl" 参数
   make -j$(nproc)
   ```

   此时我们可以确认 QEMU 的版本：

   ```shell
   qemu-system-riscv64 --version
   qemu-riscv64 --version
   ```

5. **安装GDB调试**

   额外环境要求：
   ```shell
   sudo apt-get install gcc g++
   sudo apt-get install tmux   
   ```
	安装环境依赖

    ```shell
    sudo apt-get install libncurses5-dev python python-dev texinfo 		libreadline-dev
    ```

   下载gdb并解压
   ```shell
   wget https://mirrors.tuna.tsinghua.edu.cn/gnu/gdb/gdb-10.1.tar.xz
   tar -xvf gdb-10.1.tar.xz
   ```

   安装gdb
   ```shell
   cd gdb-10.1
   mkdir build
   cd build
   ../configure --prefix=/usr/local --with-python=/usr/bin/python --target=riscv64-unknown-elf --enable-tui=yes
   make -j$(nproc)
   make install
   ```

6. **额外要求**  
   
   出于某些原因，我们全程使用 release 模式进行构建。为了正常进行调试，请确认各项目（如 os , user 和 easy-fs ）的 Cargo.toml 中包含如下配置：
   
   ```shell
   [profile.release]
   debug = true
   ```



​	



