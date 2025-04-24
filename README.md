# Degrees of Management

English | [中文](README-ZH.md)

Combine games, images, and mods through preset methods.

## Quick Start
Run the program, and the configuration file `config.toml` will be automatically created on the first run.

After completing the configuration below, visit http://localhost:3000 to access the main interface. Port can be configured through the `port` field, default is 3000.

The `data_dir` field configures the folder where the data files are nested, default is `data`.

### Data Folders
The data folder by default contains `index`, `layer`, `mod`, `instance`, `save`, wrapped by game's id.

At least one game should be defined in `config.toml`, and the `id` field in the configuration file should be consistent with the folder name.

`````toml
data_dir = "data"

[game_def.dol]
use_mods = true

[game_def.other]
use_mods = false
`````

#### Index
This directory is used to store the main game files {version}.html, which are shared game files. The file name without the .html extension is its `id`.

#### Layer
This directory is used to store other types of files such as `img/**`. The name of each folder is its `id`.

#### Mod
This directory is used to store mod files requested by ModLoader. The file name without the .zip extension is its `id`.

#### Instance
This directory stores configuration files, each file is an independent configuration:

#### Save
This directory stores save files synchronized from the web during runtime. 
Under the 'Cloud' tab appended in 'SAVE' page you can export save code to server and can load like save code as well. 
This feature is inspired by https://github.com/ZB94/dol_save_server and modified from its implementation.

**The save folders are bind to Instance ID, make sure not to change it very often.**

````json
{
  "id": "The ID of this instance, ensure it is unique",
  "name": "The display name of this instance",
  "index": "The ID of the main file (Index)",
  "layers": [
    "Layer IDs stored in an array",
    "The later the layer in this list, the higher its priority in the overlay relationship"
  ],
  "mods": [
    "Mod IDs stored in an array", 
    "Automatically loaded when accessing the game, the order is the loading order"
  ]
}
````

Here is an example:

The structure of the `data\{game_id}` folder is as follows:
````
'{game_id}'
├── index
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

The content of the `Instance.json` file is as follows:
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

Finally, when accessing the game, it will be combined into an instance named `Primitive`, accessible at the path `/play/{game_id}/1.0/index`.

When loading image files, it will first try to load from `SomeImagePatch`, then from `GameOriginalImage`. The mod will load the `I18N` mod.

**Note: All references fields in index, layers, mods do not contain extension names.**

## Build

If you need to modify the save-sync-integration mod used for synchronizing saves, execute the `pack` task, which will automatically package the mod and copy it to the server resource folder.
Packaging requires additional `dist-insertTools`, see the official repository of ModLoader for details.

For the server, simply execute `cargo build --release`.
