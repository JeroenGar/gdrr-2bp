# GDRR-2BP ![workflow](https://github.com/JeroenGar/gdrr-2bp/actions/workflows/rust.yml/badge.svg)[![DOI](https://zenodo.org/badge/doi/10.1016/j.ejor.2021.11.031.svg)](https://doi.org/10.1016/j.ejor.2021.11.031)

**This repo contains a Rust 🦀 implementation of the algorithm described in [a goal-driven ruin and recreate heuristic for the 2D variable-sized bin packing problem with guillotine constraints]( https://www.sciencedirect.com/science/article/abs/pii/S0377221721009826).**

The code used in the experiments in the paper was originally written in Java.
Due to contractual obligations, this codebase can unfortunately not be made public.
However, this reimplementation closely resembles the original design philosophy and datastructures.
It is also much (50-100x) faster than the implementation used for the benchmarks in the paper.

If this repository helps you in any way, please let me know.
I'm very interested to know whether this work is useful to anyone.

# Features

In case there are sufficient bins to pack all items, the algorithm will minimize the total bin cost required to pack all items (complete solution).
If this is not the case, it will return the solution closest to being complete (with the highest total area of packed items).

The algorithm currently has support for:
- [x] variable-sized (heterogeneous) bins
- [x] 90° rotation of items
- [x] can handle instances with insufficient bins to produce all items
- [x] configurable cost and stock quantity per bin type
- [x] configurable maximum number of guillotine stages

# How to use

## Requirements
- Rust >= 1.62

## CLI

General usage:
```bash
cargo run --release  \
    [path to input JSON] \
    [path to config JSON] \
    [path to write result JSON (optional)] \
    [path to write result HTML (optional)]
```
Concrete example:
```bash
cargo run --release \
    examples/large_example_input.json \
    examples/config.json \
    examples/large_example_result.json \
    examples/large_example_result.html
```

Make sure to include the `--release` flag to build the optimized version of the binary. 
Omitting the flag not only leads to an unoptimized binary but also enables many (highly costly) assertions that validate the correctness of the algorithm (for use during debugging).

## Input JSON

The input problem files are using the same JSON format as used in [OR-Datasets](https://github.com/Oscar-Oliveira/OR-Datasets/tree/master/Cutting-and-Packing/2D) repository by [
Óscar Oliveira](https://github.com/Oscar-Oliveira).
Any file from this repository, or other files using this format, should work. 

Two examples are provided in the [examples](examples/) folder.

## Config JSON

The config file contains all configurable parameters of the algorithm.
A detailed explanation of most of these parameters can be found in the paper.

```javascript
{
    "maxRunTime": 600, //maximum allowed runtime of the algorithm in seconds
    "nThreads": 4, //number of threads to use
    "rotationAllowed": true, //if true, 90 degree rotation of parts is allowed (2BP|R|G), false otherwise (2BP|O|G)
    "avgNodesRemoved": 6, //average number of removed nodes per iteration (μ)
    "blinkRate": 0.01, //blink rate (β)
    "leftoverValuationPower": 2, //exponent used for the valuation of leftover nodes (α)
    "historyLength": 500, //late-acceptance history length (Lh)
    "sheetValuationMode": "area", //defines how the sheets are valued (area or cost)
}
```
If `sheetValuationMode` is set to `cost`, the algorithm values each sheet based on the cost field in the input JSON.
In `area` mode, the cost field is ignored and the value of each sheet is its area. 
For maximum usage optimization, set the `sheetValuationMode` to `area`.

In addition `maxRRIterations` can also be defined. 
If provided, the algorithm will run until the predefined number of iterations is reached.   
Both `maxRRIterations` and `maxRunTime` fields are optional. 
The algorithm will continue execution until either, one of the termination conditions (defined in the config json) is reached, or it is manually terminated (CTRL+C). 

The `maxStages` field (optional) can be used to limit the number stages the guillotine saw can make.
A value of `"maxStages": 2` will ensure that all patterns can be cut with at most 2 rotations of the guillotine saw.

Configuring more than 1 thread for instances with only a single type of bin won't make much of an improvement to the end result.
On the contrary, many threads will result in a reduction of iterations/s per individual thread. 
Which, in turn, can lead to increased runtimes to reach the same solution quality.

If the instance, however, contains multiple types of bins, it is highly recommended to use multiple threads.
The diversity in bins used by the threads will usually result in a higher overall solution quality.

An example config file can be found in the [examples](examples/) folder.

## Output
Both a JSON and a visual (HTML) representation of the solution can be generated. 

### JSON

The output JSON format is an extension of the input format.
All `Items` and `Objects` now contain a `Reference` field, assigning a unique ID to every item/object. 
The root object has been extended with the fields `CuttingPatterns` and `Statistics`.

`CuttingPatterns` contain a hierarchical representation of all the cutting patterns which are part of the final solution. 
A PDF which explains the format can be found [here](doc/Solution_Files_Documentation_GDRR.pdf). 
`Statistics` contains additional information such as the average bin usage, total runtime etc.  

Examples can be found in the [examples](examples/) folder.

### HTML

In addition to the JSON solution, a visual representation of the final solution can be generated in the form of an HTML file. 

Examples can be found in the [examples](examples/) folder.

## Console

During the optimization, improving solutions are logged to the console (among other things). 
Both `complete` and `incomplete` solutions are reported.

Example:
```
[00:00:13]	[T3]	<incomplete>	(usage: 96.467%, p_incl: 99.992%, sheets: 297, mat: 60112800000000)
[00:00:13]	[T3]	<incomplete>	(usage: 96.469%, p_incl: 99.994%, sheets: 297, mat: 60112800000000)
[00:00:13]	[T3]	<incomplete>	(usage: 96.470%, p_incl: 99.995%, sheets: 297, mat: 60112800000000)
[00:00:13]	[T3]	<incomplete>	(usage: 96.472%, p_incl: 99.997%, sheets: 297, mat: 60112800000000)
[00:00:14]	[T3]	<complete>      (usage: 96.475%, p_incl: 100.000%, sheets: 297, mat: 60112800000000)
[00:00:14]	[T2]	<incomplete>	(usage: 96.492%, p_incl: 99.681%, sheets: 296, mat: 59910400000000)
[00:00:14]	[T2]	<incomplete>	(usage: 96.493%, p_incl: 99.682%, sheets: 296, mat: 59910400000000)
```
| Value          | Explanation                                                                  |
|----------------|------------------------------------------------------------------------------|
| [HH:MM:SS]     | Time passed since the start of the optimization                              |
| [T_]           | Thread that found the solution                                               |
| <(in)complete> | Denotes whether the solution contains all required items or is incomplete    |
| usage          | (total included item area) / (total used bin area) * 100                     |
| p_incl         | (total included item area) / (total required item area) * 100                |
| sheets         | Number of bins used in the solution                                          |
| mat            | Total value of the bins used in the solution                 

## Acknowledgements

This project began development at [KU Leuven](https://www.kuleuven.be/english/) and was funded by [Research Foundation - Flanders (FWO)](https://www.fwo.be/en/) (grant number: 1S71222N).

<img src="https://upload.wikimedia.org/wikipedia/commons/9/97/Fonds_Wetenschappelijk_Onderzoek_logo_2024.svg" height="50px" alt="FWO logo">
&nbsp;
<img src="https://upload.wikimedia.org/wikipedia/commons/4/49/KU_Leuven_logo.svg" height="50px" alt="KU Leuven logo">
