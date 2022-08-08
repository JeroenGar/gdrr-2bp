# GDRR-2BP ![workflow](https://github.com/JeroenGar/gdrr-2bp/actions/workflows/rust.yml/badge.svg)

**A Rust implementation of [a goal-driven ruin and recreate heuristic for the 2D variable-sized bin packing problem with guillotine constraints]( https://www.sciencedirect.com/science/article/abs/pii/S0377221721009826).**

The code used for the experiments in the paper was originally written in Java.
This code can, due to contractual requirements, unfortunately not be made public.
However, this reimplementation in Rust closely resembles the original design philosophy and datastructures.
It is also multiple times faster than the Java implementation.

If this repository helps you in any way, please let me know. 
I'm very interested to know whether this repo is useful to anyone.

# Features

The algorithm will minimize the total bin cost required to pack all items.
In case there are insufficient bins to pack all items, the algorithm will return the solution with the maximum total area of parts packed (max usage of the bins).

The algorithm currently has support for:
- [x] variable-sized (heterogeneous) bins
- [x] 90° rotation of items
- [x] insufficient bins to produce all items
- [x] individually configurable cost per bin

# How to use

## Input JSON

The input problem files are using the same JSON format as used in [OR-Datasets](https://github.com/Oscar-Oliveira/OR-Datasets/tree/master/Cutting-and-Packing/2D) repository by [
Óscar Oliveira](https://github.com/Oscar-Oliveira).
Any file from this repository, or files using the same format, should work.  
Two examples are provided in the [examples](examples/) folder.

## Config JSON

The config file contains all configurable parameters of the algorithm.
A detailed explanation of most of these parameters can be found in the paper.

```javascript
{
    "maxRunTime": 600, //maximum allowed runtime of the algorithm in seconds
    "nThreads": 1, //number of threads to use
    "rotationAllowed": true, //if true, 90 degree rotation of parts is allowed (2BP|R|G), false otherwise (2BP|O|G)
    "avgNodesRemoved": 6, //average number of removed nodes per iteration (μ)
    "blinkRate": 0.01, //blink rate (β)
    "leftoverValuationPower": 2, //exponent used for the valuation of leftover nodes (α)
    "historyLength": 500, //late-acceptance history length (Lh)
}
```
In addition `maxRRIterations` can also be defined. 
If provided, the algorithm will run until the predefined number of iterations is reached.   
Both `maxRRIterations` and `maxRunTime` fields are optional. 
If no termination condition is provided the algorithm will run until an interrupt signal (CTRL+C) is generated.  
An example config file can be found in the [examples](examples/) folder.

## Output
Both a JSON and a visual (HTML) representation of the solution can be generated. 

### JSON

The output JSON format is an extension of the input format. 
All `Items` and `Objects` now contain a `Reference` field, assigning a unique ID to every item/object. 
The root object has been extended with the fields `CuttingPatterns` and `Statistics`. 
`CuttingPatterns` contain a hierarchical representation of all the cutting patterns which are part of the final solution. 
An explanation can be found [here](doc/Solution_Files_Documentation_GDRR.pdf). 
`Statistics` contains information such as the average bin usage, total runtime etc.  
Examples can be found in the [examples](examples/) folder.

### HTML

In addition to the JSON solution, the algorithm can also output the final solution in a visual format in the form of an HTML file.  
Examples can be found in the [examples](examples/) folder.

## CLI

General usage:
```bash
cargo run --release  \
    [path to input JSON] \
    [path to config JSON] \
    [path to write result JSON to (optional)] \
    [path to write result HTML to (optional)]
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
Omitting the flag will result in an unoptimized binary which contains a lot of (expensive) assertions. 
The algorithm will continue execution until either one of the termination conditions (defined in the config json) are reached, or it is manually terminated (CTRL+C). 

