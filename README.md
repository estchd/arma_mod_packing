# Arma Mod Packing Tool

## About

This tool is designed to ease development of Arma 3 Mods by providing a one-stop shop solution for packaging and compiling your mod.
Its main goal is to enable you to keep your mods files in a workable format while giving you the ability to quickly package and test by automatically packaging the mod for you without you having to manually convert all the files into an engine usable format first.

As a secondary goal, the tool gives you the ability to take existing mods and unpack them into workable file formats in one fell swoop. NOTE: Please respect other mod authors intellectual property. While this tool can be used to breach copyright regulations, it is intended for research purposes or to restore your work files from an existing release should you ever have the need. Please do not use this tool to steal assets from mods that aren't clearly marked as to allow you to do so.

Additionally, an important goal of this tool is to improve upon the performance of existing tools, giving you the ability to quickly iterate upon your mod creations.

Finally, this tool aims to provide better error messaging than the ```CfgConvert failed: -1``` style messages of the standard Arma Toolset.

## Current state and future plans

In its current state, this tool fulfills its main goals of packing and unpacking mods. It currently relies on the standard Arma toolset and some other community tools for this. This means that the performance is currently comparable to just using the tools directly and the error messaging, while a bit better than using the standard tools, is still not quite where it should be.

In the future, I aim to replace the standard and community tools used by custom written ones to have full control over both performance and error handling as well as to not require users of this tool to install any other programs before use.

In the end, I would like this tool to grow into more or less a full-fledged, all-in-one Arma modding environment but that is a very far-off goal. For now, it will stay a small console tool for a single use-case.

## Installation

### Prerequisites

Before installing and using this tool, you will have to install a few prerequisites that are needed in order for this tool to perform some of its tasks.

You will need:

- The "Arma 3 Tools" suite installed 
  - preferably from steam but it doesn't really matter
  - You only need the tools themselves, no need to set a ```P:``` drive
- [This pbo manager tool](https://github.com/winseros/pboman3)

### Installation

To install the tool, follow these steps:

1. Download the latest release from the GitHub page
2. Unpack the tool at a location of your choice
3. If wanted, add the tool location to your ```$PATH``` variable
4. Follow the Configuring tool paths section

### Configuring tool paths

The tool uses a ```path.json``` file to tell it where to find the other programs it needs.
If you do not specify the location of the path.json file, the tool will use a path.json file in the folder where the tool itself is located.
If there is no ``path.json`` there, it will automatically generate a default one once run for the first time. NOTE: If you specify a path.json path through the command line, that file must exist or the tool will fail to run.

The default ```path.json``` file assumes that all tools are located in an ```external_tools``` folder in the program directory.

If you put add all the required programs to your ```$PATH``` variable, you can also simply specify the exe name as the program path though doing so may pollute your ```$PATH``` and be undesirable.

As for what programs the different fields should point to:

- paa_converter_path
  - should point to the ImageToPAA.exe tool
  - commonly found at ```[your arma tools foler]/ImageToPAA/ImageToPAA.exe```
- rvmat_converter_path
  - should point to the CfgConvert.exe tool
  - commonly found at ```[your arma tools folder]/CfgConvert/CfgConvert.exe```
- config_converter_path
  - should point to the CfgConvert.exe tool
  - commonly found at ```[your arma tools folder]/CfgConvert/CfgConvert.exe```
- pbo_packer_path
  - should point to the pboc.exe
  - found in the installation directory of the PBO Manager tool
- pbo_signer_path
  - should point to the DSSignFile.exe
  - commonly found at ```[your arma tools folder]/DSSignFile/DSSignFile.exe```

## Usage

This tool can operate in one of two modes, either packing a mod from common file formats or unpacking it from an Arma ready format.

### Packing a mod
Command: ```arma_tools_mod_packing.exe --source <SOURCE> --destination <DESTINATION> --pack [--path_json <PATH_JSON>]```

```SOURCE```: The source folder with all your mods files. This should be the folder containing the mod.cpp file.

```DESTINATION```: The destination folder where the packed files are copied to. After packing, this folder will directly contain the mod.cpp file and the addons folder.

Packing a mod requires a few additional files to be included in your mod. These files contain additional configuration information so that you can specify what files are included in your mod, what files are not converted into an Arma ready format and what folders are packed into ```.pbo``` files. But do not worry, none of these configuration files will end up inside the finished mod.

#### Preparing a mod for packing

To prepare a mod for packing, first structure your mod project directory in the same way that you want to have in your final mod. After that, follow these steps:
1. Copy the default ```.modignore``` from the release files of this tool into the root folder of your mod. Extend as needed.
2. Copy the default ```.convertignore``` from the release files of this tool into the root folder of your mod. Extend as needed.
3. Copy the default ```pbo.json``` from the release files of this tool into every folder you want packed into a ```.pbo``` file. Set the ```pbo_prefix``` and other fields or headers as needed.
4. Copy the default ```key.json``` from the release files of this tool into every folder with a ```pbo.json``` that you want signed. Change the authority name to the authority name you specified when creating your key pair.
5. Copy the ```.bikey``` and ```.biprivkey``` files for every key pair you want to use to sign to the ```keys``` directory in your mods project directory.

#### pbo.json

The ```pbo.json``` file specifies what folders are packed into ```.pbo``` files and how that packing is done.
The format for the ```pbo.json``` is specified [here](https://github.com/winseros/pboman3/blob/develop/doc/pbo_json.md).

NOTE: Only folders containing a ```pbo.json``` file are packed. All other folders (unless specified in ```.modignore```) are copied as-is into the final mod. The files within these folders are converted as usual.

NOTE: Only the topmost ```pbo.json``` in any folder and its sub-folders will be packed. Any ```pbo.json``` found in a sub-folder of a folder with a ```pbo.json``` will be included in the packed ```.pbo````file as-is.

#### key.json

The ```key.json``` file is used to tell the tool, which ```.pbo``` to sign, and which key files to use for that purpose.
Any folder containing a ```key.json``` that is packed into a ```.pbo``` is signed with the key specified in that file.

NOTE: Only ```key.json``` files that are located in the same folder as a ```pbo.json``` are considered, all other ```key.json``` files are copied as-is into the resulting packed ```.pbo``` file (or the corresponding folder in the mod if not inside a ```.pbo```).

#### .modignore

The ```.modignore``` file lets you specify, which files are ignored completely. Fies specified by the ```.modignore``` file are not copied into the output directory, converted or included in any packed ```.pbo``` file.

The ```.modignore``` file follows the [.gitignore](https://git-scm.com/docs/gitignore) file format.

A common usage for the ```.modignore``` file would be excluding development files like documentation, project configuration or similar files that you do not want to end up in the final mod.

#### .convertignore

The ```.convertignore``` file lets you specify, which files are not converted into Arma ready format. But are copied into the output directory as-is and are included in packed ```.pbo``` files in the same manner.

The ```.convertignore``` file follows the [.gitignore](https://git-scm.com/docs/gitignore) file format.

A common usage for the ```.convertignore``` file would be excluding additional files like ```README.md``` or similar, that you do want to be included in the final mod or that should not or cannot be converted into Arma ready formats.

NOTE: There are some files that will probably always have to be included like the ```pbo.json``` and ```key.json``` files. 
There are also some file formats like audio files or ```.p3d``` files that are commonly included in mods but either have no conversion or cannot be converted by the tool yet.
For now, use the ```.convertignore``` included in the release files and extend it as needed.

### Unpacking a mod

Command: ```arma_tools_mod_packing.exe --source <SOURCE> --destination <DESTINATION> --unpack [--path_json <PATH_JSON>]```

```SOURCE```: The source folder with the packed mod files. This should be the folder containing the mod.cpp file.

```DESTINATION```: The destination folder where the unpacked files are copied to. After packing, this folder will directly contain the mod.cpp file and the addons folder.

For unpacking, no additional configuration is needed. The tool should work directly with any mods that can be loaded into Arma, as they are.
