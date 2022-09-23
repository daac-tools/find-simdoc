# All pairs similarity search on binary sketches in the Hamming space

This library provides a fast and compact all pairs similarity search (or *similarity self-join*)
on binary sketches in the Hamming space.
The algorithm employs a modified variant of the [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html),
a combination of the [multiple sorting](https://doi.org/10.1007/s10115-009-0271-6)
and the [multi-index approach](https://doi.org/10.1109/TKDE.2019.2899597).

This library is a part of [find-simdoc](https://github.com/legalforce-research/find-simdoc).
