# Simple Path Tracer

A simple path tracer written in Rust.

It's a playground for me to learn something like BSSRDF and volume rendering, and also to discover the effect of some samplers and filters.

Project structure is similar to that in pbrt.

## Implemented Features

* Texture mapping (support tiling and offset) and mipmap
* Importance sampling to HDR(`.exr`) environment map using atlas method
* Surface area hierarchy
* Multiple importance sampling
* Simple microfacet material (GGX NDF and Smith separable visible term, importance sampling w.r.t GGX NDF)
* Microfacet glass material
* Homogeneous medium with Henyey-Greenstein phase function
* BSSRDF with normalized diffusion profile
* Glinty surface material ([Position-Normal Distributions for Efficient Rendering of Specular Microstructure, Yan et al. 2016](https://sites.cs.ucsb.edu/~lingqi/publications/paper_glints2.pdf))
* Cubic Bézier surface
  * Bézier clipping (default)
  * Newton's iteration (feature `bezier_ni`)
* Catmull-Clark subdivision surface
  * Use feature adaptive subdivision
  * Boundary, creases are partially supported
  * Texture mapping are not supported