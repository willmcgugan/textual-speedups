# Textual Speedups

Optional Rust speedups for [Textual](https://github.com/textualize/textual).

## How to use

To use textual-speedups, install it in to the same environment as your Textual app:

```bash
pip install textual-speedups
```

Or add `textual-speedups` to your dependences.
Textual will use the speedups library automatically.

If you ever want to disable speedups in case of a bug or you want to compare with and without speedups, then set `TEXTUAL_SPEEDUPS=0`.

For example:

```bash
TEXTUAL_SPEEDUPS=0 python -m textual
```

## What is faster?

Currently, the classes in geometry.py have a Rust implementation.
This includes `Offset`, `Size`, `Region`, and `Spacing`.
These classes are used a lot internally when updating layout in particular.

## How much faster?

That is yet to be determined.
A little profiling suggests that pretty much all methods are several orders of magnitude faster than the pure-Python versions.
This should add up to a respectable improvement, but until I've written a benchmark tool I won't be able to quantify that.

Note that unless your Textual app is particularly complex, you might not even notice a difference!

## Bugs?

All tests pass with speedups enabled, and every app I have tested it against.
However, it is possibly (even likely) there are edge cases remaining which may result in crashes or subtle differences.
