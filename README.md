# pzexeconfig-installer
Config that is downloaded originates here: https://gist.github.com/aoqia194/f93a6d9cdfd66388c46ada22d067b058

## Usage
The *install* binary is the preferred method, as it is never deleted by Steam or even game uninstall. This means you will never have to do it again, unless the file is manually deleted somehow.

`install` bin downloads the config to the Project Zomboid install directory. Will never get deleted by Steam or by uninstall but requires you to add `-pzexeconfig ProjectZomboid64aoqia.json` to the launch options.

`overwrite` bin downloads the config to the Project Zomboid directory and overwrites the original `ProjectZomboid64.json` config. This means that it will be replaced when Steam does it's CRC checks.
