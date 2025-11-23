# The RollKit Grammar

RollKit is a dice-rolling library that allows users to define and evaluate complex dice expressions. This document outlines the grammar used in RollKit expressions.

## Basic Components

A RollKit expression is an expression composed of values and operators described below. It looks similar to expressions in many programming languages, with some additions for dice rolling.

Here are some examples of valid RollKit expressions: `3d6 + 2`, `4d10kh3 - 1`, `2d{1,2,3} * 5`, `5 + (2d8dl1)`.

### Values

RollKit supports two primary types of values: 

- **Integers**: `1`, `42`, `-7`, `3 + 4`.
- **Lists**: `{1, 2, 3}`, `{4, 5, 6, 7}`, `3d6`.

Integers are just integers we all know and love. Lists are ordered collections of 0 or more integers, and they can be created by either operators that return lists, such as dice rolls, or by one of the two list literals:

- **Explicit List Literal**: A list defined by curly braces, e.g., `{1, 2, 3}`.
- **Range List Literal**: A list defined by a range and an optional step, e.g., `[1, 3]` for `{1, 2, 3}`, `[5, 15, 5]` for `{5, 10, 15}`. Note that:
    - The start and end values are inclusive.
    - The start value can be greater than the end value for descending ranges, e.g., `[10, 5]` for `{10, 9, 8, 7, 6, 5}`.
    - The sign of the step value is ignored, a step of `0` is invalid, e.g., both `[1, 10, 2]` and `[1, 10, -2]` produce `{1, 3, 5, 7, 9}`.

Lists are further devided into two subtypes: **Strong Lists** and **Normal Lists**. The only difference is that when performing arithmetic and comparison operations, Normal Lists will be automatically reduced to their sum, while Strong Lists will not and the operation will be performed element-wise. **Strong Lists** are created by wrapping a Normal List with braces, e.g., `{{1, 2, 3}}` or `{3d6}`.

### Operators

RollKit supports a variety of operators, and here is the complete list, with their precedence (from highest to lowest):

- Rolling Operators:
    - Dice Roll: `d: Integer x Integer | List -> List`, right associative, e.g., `3d6` rolls three six-sided dice, `2d{1,2,3}` rolls two dice with faces 1, 2, and 3.
    - Dice Keep/Drop Highest/Lowest: `kh/kl/dh/dl: List x Integer -> List`, left associative, e.g., `4d6kh3` rolls four six-sided dice and keeps the highest three.
- Arithmetic Operators:
    - Multiplication: `*: Integer x Integer -> Integer`, left associative, e.g., `3 * 4` results in `12`.
    - Addition and Subtraction: `+/ -: Integer x Integer -> Integer`, left associative, e.g., `5 + 2` results in `7`, `10 - 3` results in `7`.
- Comparison Operators:
    - Comparison: `== / != / < / <= / > / >=: Integer x Integer -> Integer`, left associative, e.g., `5 > 3` results in `1` (true), `2 == 4` results in `0` (false).

Parentheses `()` can be used to group expressions and override the default precedence, as in other programming languages.

Function calls are also supported, with the syntax: `functionName(arg1, arg2, ...)`, where `functionName` is the name of the function and `arg1`, `arg2`, etc. are the arguments passed to the function.
