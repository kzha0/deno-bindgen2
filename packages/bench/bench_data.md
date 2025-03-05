# Benchmark data

Average processing time using custom test suite with work arounds for the proc-macro API limitation

| test | deno_bindgen | deno-bindgen2 |
|-|-|-|
|  1 | 298,729.93 ns | 19,504.45 ns |
|  2 | 300,780.80 | 20,043.88 |
|  3 | 300,973.38 | 19,663.09 |
|  4 | 298,998.10 | 20,361.22 |
|  5 | 299,441.12 | 20,263.94 |
|  6 | 298,473.53 | 19,974.75 |
|  7 | 297,670.50 | 20,170.54 |
|  8 | 300,651.90 | 19,746.59 |
|  9 | 298,244.10 | 19,578.11 |
| 10 | 300,721.25 | 19,789.03 |
| **average** | **299,468.46** | **19,909.56** |

Difference: 279,558.90 ns \
Change: -93.35%
