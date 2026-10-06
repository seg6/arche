//! Levels, with move-optimal solutions (checked by the tests).

pub struct Level {
    pub name: &'static str,
    pub map: &'static str,
    /// u/d/l/r; its length is the par.
    pub solution: &'static str,
}

pub const LEVELS: &[Level] = &[
    Level {
        name: "first day",
        map: "
#######
#@ $ .#
#######",
        solution: "rrr",
    },
    Level {
        name: "two step",
        map: "
######
#    #
# $$ #
#@ ..#
######",
        solution: "uurdldrururd",
    },
    Level {
        name: "pinwheel",
        map: "
#######
#  .  #
# #$# #
#.$@$.#
# #$# #
#  .  #
#######",
        solution: "uddulrr",
    },
    Level {
        name: "zigzag",
        map: "
########
#  .   #
# $##$ #
#.  @  #
###$## #
  #.   #
  ######",
        solution: "rruuldrdllldululur",
    },
    Level {
        name: "loading dock",
        map: "
########
#      #
# .**$@#
#      #
#####  #
    ####",
        solution: "ulldldruurrdllrrddlurul",
    },
    Level {
        name: "inventory",
        map: "
  ####
###  ####
#     $ #
# #  #$ #
# . .#@ #
#########",
        solution: "ruullluldrrrrddlurulllddllluurrdrdluuurdd",
    },
    Level {
        name: "courtyard",
        map: "
  ######
  #    #
### ## #
#  $ $ ##
# #.*.  #
#  $  # #
###@   .#
  #######",
        solution: "rururuuulllddrrlluurrrddllddrdrruululllllddrrdrudrr",
    },
];
