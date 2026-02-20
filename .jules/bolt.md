## 2024-05-23 - [Audio Processing Optimization]
**Learning:** In tight loops like audio resampling (processing millions of samples/sec), replacing floating-point division (`i / ratio`) with addition (`pos += step`) yielded a measurable ~28% performance improvement.
**Action:** Always look for division operations inside tight loops and replace them with accumulation where possible.
