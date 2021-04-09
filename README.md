# Simple Path Tracer

A simple path tracer written in Rust.

It's a playground for me to learn something like BSSRDF and volume rendering, and also to discover the effect of some samplers and filters.

Project structure is similar to that in pbrt.

## Implemented Features

* Multiple Importance Sampling
* Simple Microfacet Material (just use similar way to that in real-time rendering, GGX NDF and Smith separable visible term)
* Medium Rendering (simple homogeneous medium) (not finished yet ...)