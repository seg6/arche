# coding: utf-8
"""Levels, with move-optimal solutions found by board.solve()."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Level:
    name: str
    map: str
    solution: str   # u/d/l/r, also the par


LEVELS = [
    Level('first day', """
#######
#@ $ .#
#######
""", 'rrr'),

    Level('two step', """
######
#    #
# $$ #
#@ ..#
######
""", 'uurdldrururd'),

    Level('pinwheel', """
#######
#  .  #
# #$# #
#.$@$.#
# #$# #
#  .  #
#######
""", 'uddulrr'),

    Level('zigzag', """
########
#  .   #
# $##$ #
#.  @  #
###$## #
  #.   #
  ######
""", 'rruuldrdllldululur'),

    Level('loading dock', """
########
#      #
# .**$@#
#      #
#####  #
    ####
""", 'ulldldruurrdllrrddlurul'),

    Level('inventory', """
  ####
###  ####
#     $ #
# #  #$ #
# . .#@ #
#########
""", 'ruullluldrrrrddlurulllddllluurrdrdluuurdd'),

    Level('courtyard', """
  ######
  #    #
### ## #
#  $ $ ##
# #.*.  #
#  $  # #
###@   .#
  #######
""", 'rururuuulllddrrlluurrrddllddrdrruululllllddrrdrudrr'),
]
