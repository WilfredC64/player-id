# Player-ID

[![Rust](https://github.com/WilfredC64/player-id/actions/workflows/rust.yml/badge.svg)](https://github.com/WilfredC64/player-id/actions/workflows/rust.yml)

Player-ID a.k.a. C64 Music Player Identifier (PI) is a cross-platform utility to
identify Commodore 64 music players in SID files.

Player-ID is inspired by the [SIDId](https://github.com/cadaver/sidid/)
tool written by Cadaver of Covert Bitops. <nobr>Player-ID</nobr> makes
use of all available cores of the CPU and uses the BNDM (Backward Nondeterministic Dawg Matching) search algorithm to
search through files very quickly.

## Development

To build the source code you'll need to have 
[Rust](https://www.rust-lang.org/) installed.

For building:

```
cargo build --release
```

## Usage

Usage: player-id &lt;options&gt; &lt;file_path_pattern&gt;

### &lt;file_path_pattern&gt;

> The file_path_pattern can be any SID or other filename. You can use
wildcards to process multiple SID and PRG files. You may want to use the <nobr>-s</nobr>
option to process sub folders as well. If you have spaces in the filename or
in the folder name then surround the folder and filename with double quotes.

Examples:
* *.sid
* tune?.sid
* "C:\\my c64 music collection\\sids\\*.sid"
* C:\\HVSC\\C64Music\\*.sid
* ~"/HVSC/C64Music/*.sid"

### &lt;options&gt;

**-c{max_threads}**: set the maximum CPU threads to be used [Default is all]

> Use the <nobr>-c</nobr> option to limit CPU thread usage. By default, it will use all
available CPU threads. This tool is optimized for running on multiple CPUs or
on CPUs with multiple cores. The more CPU threads it can use, the faster the
searches will be.

**-f{config_file}**: config file to use [Default SIDIDCFG environment variable / sidid.cfg file]

> Use the -f option if you want to use a different config file than the
default.
<br>If the config file is not specified by the -f option, then it will try to
find the <nobr>"sidid.cfg"</nobr> file via the **SIDIDCFG** environment variable setting. If
the variable is not present then it will try to find the <nobr>"sidid.cfg"</nobr> file in
the same directory as where <nobr>player-id</nobr> is located.

**-h**: scan HVSC location [Uses HVSC environment variable for HVSC path]

> Use the <nobr>-h</nobr> option to scan the HVSC location for known players. The HVSC
location needs to be specified via the environment variable **HVSC**. Using this option will also
set the file_path_pattern to *.sid when it is not specified, and it will also search through
subdirectories.

**-n**: show player info [use together with -p option]

> Use the <nobr>-n</nobr> option to show the player info, if available. You'll need to
specify the player ID with the <nobr>-p</nobr> option.

**-m**: scan for multiple signatures

> Use the <nobr>-m</nobr> option if you want to scan files for multiple signatures. Some SID
files contain multiple players. When the <nobr>-m</nobr> option is not specified only the
first found player will be returned. The first found player is dependent on
the order of the player signatures in the sidid.cfg file.
When a player is found multiple times in the file, the <nobr>-m</nobr> option will only
return the player name once.

**-o**: list only unidentified files

> Use the <nobr>-o</nobr> option if you're only interested in a list of files that could not
be identified.

**-p{player name}**: scan only for specific player name

> Use the <nobr>-p</nobr> option if you only want to scan for a specific player name. For
the list of player names you can check the sidid.cfg file. A player name can't
contain spaces and is case-insensitive.

**-s**: include subdirectories

> Use the <nobr>-s</nobr> option if you want to search multiple files through multiple
subdirectories. When you use the index via the -h option, then you don't have
to specify the <nobr>-s</nobr> option.

**-t**: truncate filenames

> Use the <nobr>-t</nobr> option to truncate the filenames so that the signatures found
column is always at the same column. When the -t isn't used, <nobr>player-id</nobr> will
set the signatures found column based on the longest filename.

**-u**: list also unidentified files

> Use the <nobr>-u</nobr> option if you're also interested in files that could not be
identified. All files that are scanned will be listed.

**-v**: verify signatures

> Use the -v option if you want to verify signatures for errors. This option is
useful when you create your own signatures. This option will also verify the
info file <nobr>(sidid.nfo)</nobr> when it's found.

**-wn**: write signatures in new format

> Use the -wn option if you want to write a signatures file to the new file format (V2).

**-wo**: write signatures in old format

> Use the -wo option if you want to write a signatures file to the old file format (V1).

**-x**: display hexadecimal offset of signature found

> Use the -x option if you want to display the hexadecimal offset where the
signature has been found. When a signature uses an <nobr>AND/&&</nobr> token then it
will display all the offsets of the sub signatures.

## Examples

For searching through all the SID files in HVSC:

```
player-id -s "C:\HVSC\C64Music\*.sid"
```

For identifying multiple signatures in all the SID files in HVSC:

```
player-id -s -m "C:\HVSC\C64Music\*.sid"
```

For scanning HVSC:

```
player-id -h
```

For scanning HVSC and identify multiple signatures:

```
player-id -h -m
```

For scanning HVSC for a specific file pattern:

```
player-id -h Commando*.sid
```

For scanning files that include e.g. SoundMonitor player:

```
player-id -pSoundMonitor -s "C:\HVSC\C64Music\*.sid"
```

For retrieving the player info of e.g. SoundMonitor:

```
player-id -pSoundMonitor -n
```

## File Format

The signature file format specification can be found [here](/doc/Signature_File_Format.txt).

## Copyright

Player-ID &ndash; Copyright &#xa9; 2012 - 2023 by Wilfred Bos.

Signatures created by: Wilfred Bos, iAN CooG, Professor Chaos, Cadaver, Ninja, Ice00 and Yodelking.

## Licensing

The source code is licensed under the MIT license. License is available [here](/LICENSE).
