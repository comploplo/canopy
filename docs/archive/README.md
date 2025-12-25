# Archived Documentation

This directory contains historical and future-vision documentation that is not part of the current implementation.

**For current implementation, see:**

- [ARCHITECTURE.md](../ARCHITECTURE.md) - Current system architecture
- [ROADMAP.md](../ROADMAP.md) - Milestone progress and evolution

______________________________________________________________________

## Directory Structure

### `/theoretical-foundations/`

Research foundations and theoretical design documents.

| File                   | Description                                      | Status             |
| ---------------------- | ------------------------------------------------ | ------------------ |
| `THEORY.md`            | Computational linguistic foundations, DRT theory | Research reference |
| `layer3-drt-design.md` | Layer 3 DRT design specification                 | Planned for M8+    |

### `/future-vision/`

Strategic vision documents for deferred features.

| Directory                 | Description                          | Target   |
| ------------------------- | ------------------------------------ | -------- |
| `gpu-acceleration/`       | GPU-accelerated parsing architecture | M10+     |
| `neurosymbolic-ai/`       | Neural-symbolic integration plans    | Research |
| `theoretical-extensions/` | PropBank, construction grammar, etc. | M8+      |

### Root Files

| File                     | Description                        | Status               |
| ------------------------ | ---------------------------------- | -------------------- |
| `GPU_ACCELERATION.md`    | GPU acceleration strategy overview | Deferred to M10+     |
| `RESEARCH_DIRECTIONS.md` | Research ideas and directions      | Historical reference |

______________________________________________________________________

## When to Reference These Documents

- **Research context**: Understanding theoretical foundations
- **Long-term planning**: GPU acceleration, neurosymbolic AI
- **Historical context**: How the architecture evolved

## When NOT to Use These Documents

- **Current implementation details**: Use ARCHITECTURE.md instead
- **API usage**: Use implementation/ docs and crate docs
- **Performance tuning**: Use reference/performance.md
