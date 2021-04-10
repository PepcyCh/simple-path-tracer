# Simple Path Tracer

A simple path tracer written in Rust.

It's a playground for me to learn something like BSSRDF and volume rendering, and also to discover the effect of some samplers and filters.

Project structure is similar to that in pbrt.

## Implemented Features

* Surface area hierarchy
* Multiple importance sampling
* Simple microfacet material (GGX NDF and Smith separable visible term, importance sampling w.r.t GGX NDF)
* Homogeneous medium with Henyey-Greenstein phase function