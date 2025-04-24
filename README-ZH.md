# Degrees of Management

[English](README.md) | 中文

通过预设的方式组合游戏，图像与模组。

本项目仅供学习交流，研究游戏原理使用，本人不支持任何形式的公开DoL分发。

## 快速开始
执行程序，在第一次执行文件时会自动创建配置文件config.toml

在下面的配置完成后，访问 http://localhost:3000 即可访问主界面。端口可以通过`port`字段配置，默认为3000。

`data_dir`字段配置数据文件被嵌套的文件夹，默认为`data`

当然，你也可以使用Docker版本，在 'Packages' 中可以找到。

### 数据文件夹
数据文件夹默认有`index`，`layer`，`mod`，`instance`，`save`。

#### Index
该目录用于存储{version}.html的游戏主文件，用来作为共享的游戏主文件，去除.html后缀的文件名为它的`id`。

在`config.toml`中至少应该定义一个游戏，配置文件中的`id`字段应该与文件夹名称保持一致。

````toml
data_dir = "data"

[game_def.dol]
use_mods = true

[game_def.other]
use_mods = false
````

#### Layer
该目录用于存储`img/**`等其它类型的文件，每个文件夹的名字为它的`id`。

#### Mod
该目录用于请求用于ModLoader请求的模组文件，去除.zip后缀的文件名为它的`id`。

#### Instance
该目录存储配置文件，每个文件为一个独立配置：

#### Save
该目录存储运行中从网页同步的存档文件，可以在存档加载页面附加的“云存档”标签下找到上传与加载功能，与通常的存档码类似。
该功能受 https://github.com/ZB94/dol_save_server 的启发，修改自该项目中的实现。

**存档目录与Instance的ID绑定，确保不要经常修改Instance ID**

````json
{
  "id": "该实例的ID，确保唯一",
  "name": "该实例的显示名称",
  "index": "主文件(Index)的ID",
  "layers": [
    "数组形式存放的Layer ID",
    "该列表中位置越后的Layer在覆盖关系中优先级最高"
  ],
  "mods": [
    "数组形式存储的Mod ID", 
    "在访问游戏时自动加载，顺序即为加载排序"
  ]
}
````

下面是一个示例：

`data\{game_id}`文件夹结构如下：
````
'{game_id}'
├── foundation
│   ├── 1.0.html
│   ├── 1.1.html
│   └── 1.2.html
├── layer
│   ├── GameOriginalImage
│   ├── SomeImagePatch
│   └── SomeImagePatchUnused
├── mod
│   ├── I18N.zip
│   └── AnotherMod.zip
└── instance
    └── Instance.json
````

`Instance.json`文件内容如下：
````json
{
  "id": "1.0",
  "name": "Primitive",
  "index": "1.0",
  "layers": [
    "GameOriginalImage",
    "SomeImagePatch"
  ],
  "mods": [
    "I18N"
  ]
}
````

最终在访问游戏时，就会组合成一个名为Primitive的实例，生成在/play/{game_id}/1.0/index访问路径下。

加载图像文件时，优先从`SomeImagePatch`加载，然后再尝试从`GameOriginalImage`加载，模组则会加载`I18N`模组。

**注意：index，layers，mods的引用，都不带后缀名**

## 构建

如果需要修改同步存档用的save-sync-integration模组，执行`pack`任务即可，会自动打包门模组并拷贝到服务端资源文件夹。
打包需要额外的`dist-insertTools`，详见ModLoader的官方仓库。

对于服务端，直接执行`cargo build --release`即可。
