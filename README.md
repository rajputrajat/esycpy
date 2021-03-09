# esycpy

## ***WORK IN PROGRESS***

## Why?

I needed a utility to create hard-links of files rathen than copying those.
Hence this work.

Initially done the same in Powershell script, which was also at functional stage more or less.
This still require much work and possibly restructuring.

Output from > esycpy --help

---

USAGE:
    esycpy [SUBCOMMAND]
    esycpy [OPTIONS]

FLAGS:
    -h, --help      Prints help information
    -V, --version   Prints version information

OPTIONS:
    -j, --json <JSON_FILE_PATH>
                    Json file path which defines copy/move/hardlink operations
    -v, --variables <VARIABLE_NAME_VALUE_PAIR>...
                    these are optionally used in input json file,
                    multiple values can be given like this <var_name=var_value>

SUBCOMMANDS:
    copy            copy file/dir from source to destination
    hardlink        create hardlinks of file/s from source to destination
    help            Prints this message or the help of the given subcommand(s)
    move            move file/dir from source to destination

EXAMPLES:
    USING INPUT JSON FILE:
    > esycpy -j /home/example/asset_copier.json
    ------------------------- ASSET_COPIER.JSON -----------------------------
    |                                                                       |
    |    "variables_in_use": [                                              |
    |    ],                                                                 |
    |    "jobs": [                                                          |
    |        {                                                              |
    |            "todo": "hardlink",                                        |
    |            "src": "c:/Users/example/src_dir",                         |
    |            "dst": "c:/Users/example/desktop/here"                     |
    |        },                                                             |
    |        {                                                              |
    |            "todo": "hardlink",                                        |
    |            "src": "c:/Users/example/src_dir/*",                       |
    |            "dst": "c:/Users/example/documents/hlinks_all_here"        |
    |        },                                                             |
    |    ]                                                                  |
    |}                                                                      |
    -------------------------------------------------------------------------

    > esycpy -j /home/example/copier.json -v songs_dir=/home/example/songs pdfs=/home/example/study docs=/home/example/documents
    --------------------------- COPIER.JSON ---------------------------------
    |{                                                                      |
    |    "variables_in_use": [                                              |
    |        "{songs_dir}",                                                 |
    |        "{pdfs}",                                                      |
    |        "{docs}"                                                       |
    |    ],                                                                 |
    |    "jobs": [                                                          |
    |        {                                                              |
    |            "todo": "hardlink",                                        |
    |            "src": "{songs_dir}/**.ogg",                               |
    |            "dst": "/home/example/all_oggs"                            |
    |        },                                                             |
    |        {                                                              |
    |            "todo": "move",                                            |
    |            "src": "{pdfs},                                            |
    |            "dst": "/home/example/to_new_dir"                          |
    |        },                                                             |
    |        {                                                              |
    |            "todo": "copy",                                            |
    |            "src": "{docs}/*.docx",                                    |
    |            "dst": "/home/example/this_dir_docx_files"                 |
    |        },                                                             |
    |    ]                                                                  |
    |}                                                                      |
    -------------------------------------------------------------------------

    USING OPTIONS:
    1. Create hardlink of file to new_hard_link
        > esycpy hardlink -s c:/users/example/file -d c:/users/example/new_hard_link
    2. Create hardlink of dir recursively
        > esycpy hardlink -s c:/users/example/dir1 -d c:/users/example/dir_with_hlinks
    3. Move xml files from this dir to destination dir
        > esycpy move -s c:/users/example/dir2/*.xml -d c:/users/example/dir_move_in_here
    4. Copy all ogg files recursively to destination dir
        > esycpy copy /home/example/audios/**.ogg /home/example/only_oggs
