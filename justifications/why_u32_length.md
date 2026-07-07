# Why u32 for Record Length

## Decision

**Lulu-Logs uses a fixed `u32` (big-endian) to prefix the length of each FlatBuffer LogRecord in the stream.**

This means every record in a `.lulu` file or stream is encoded as:
```
+----------------+---------------------+
| u32 (4 bytes)  | FlatBuffer LogRecord |
| Length (BE)   | (variable size)      |
+----------------+---------------------+
```

## Rationale

### 1. FlatBuffers Does Not Support Varint

FlatBuffers **does not natively support varint encoding** for field sizes or table lengths. The FlatBuffers wire format uses:
- Fixed-size integers (`u8`, `u16`, `u32`, `u64`) for scalar fields.
- `u32` for vector lengths (e.g., `[ubyte]`).
- `u32` for string lengths.

While the **stream framing** (record length prefix) is not part of the FlatBuffer schema itself, **aligning with FlatBuffers' design philosophy** simplifies implementation and avoids inconsistency.

> **Reference**: [FlatBuffers Documentation - File Format](https://google.github.io/flatbuffers/md__file_format.html)

### 2. Lulu-Logs Is Not Designed for Extremely Large Records

Lulu-Logs is optimized for **high-frequency, structured logging** where:
- Most log records are **small** (typically < 1 KB).
- Records are **frequent** (thousands per second).
- **Low overhead per record** is critical.

Using `u32` for length:
- ✅ **Fixed overhead**: 4 bytes per record, regardless of size.
- ✅ **Simple parsing**: No need for varint decoding logic.
- ✅ **Predictable performance**: Constant-time length read.

Varint would:
- Add **complexity** (variable-length decoding).
- Save **only 1-3 bytes** for small records (which are already small).
- Offer **no benefit** for typical use cases.

### 3. Practical Size Limits

A `u32` length allows records up to **4 GB** in size. This is:
- **Far beyond** any realistic log record (even with large `Bytes` or `Json` payloads).
- **Sufficient** for all foreseeable use cases (sensor data, spans, scenarios, etc.).
- **Consistent** with common streaming formats (e.g., Protocol Buffers' default message size limits).

If a record exceeds 4 GB, it should be:
- Split into multiple records.
- Stored externally (e.g., as a file) with a reference in the log.

### 4. Alignment with Existing Ecosystem

Many widely adopted streaming formats use fixed-size length prefixes:
| Format | Length Prefix | Example Use Case |
|--------|---------------|------------------|
| **Protocol Buffers** | `u32` (varint-encoded) | RPC, serialization |
| **Cap'n Proto** | `u32` | Serialization |
| **MessagePack** | `u8`, `u16`, `u32` | JSON alternative |
| **Lulu-Logs** | `u32` (big-endian) | Logging |

While Protocol Buffers uses varint for the length prefix, this is because:
- Protobuf's **wire format** is varint-based.
- It is designed for **general-purpose serialization**, including very small messages.

Lulu-Logs, however, is **specialized for logging** and prioritizes:
- **Simplicity** (no varint decoding).
- **Consistency** (FlatBuffers uses fixed sizes).
- **Performance** (fixed-size reads are faster).

### 5. Big-Endian for Cross-Platform Compatibility

The length prefix uses **big-endian** (`network byte order`) to ensure:
- **Consistent behavior** across all architectures (x86, ARM, etc.).
- **No endianness conversion** needed when streaming over network or writing to files.
- **Compatibility** with standard networking practices.

### 6. Benchmark Considerations

| Approach | Overhead (bytes) | Parsing Speed | Complexity |
|----------|------------------|---------------|------------|
| `u32` (fixed) | 4 | ⚡ Fastest | ⭐ Low |
| `u16` (fixed) | 2 | ⚡ Fast | ⭐ Low (but limits to 64 KB) |
| `u8` (fixed) | 1 | ⚡ Fast | ⭐ Low (but limits to 256 B) |
| Varint | 1-5 | 🐢 Slower | ⭐⭐ Medium |

For Lulu-Logs:
- **`u32` is the sweet spot**: No practical size limit, fastest parsing, simplest code.
- **Varint would add complexity** for negligible gains (1-3 bytes saved per record).

### 7. Future-Proofing

If extremely large records (> 4 GB) ever become necessary:
1. **Switch to `u64`**: Still fixed-size, still simple.
2. **Use chunking**: Split large payloads into multiple records.
3. **External storage**: Store large data (e.g., binary blobs) separately and reference by ID.

None of these require varint.

---

## Alternatives Considered

### ❌ Varint (Like Protocol Buffers)
- **Pros**: Saves 1-3 bytes for small records.
- **Cons**:
  - FlatBuffers does not use varint, leading to inconsistency.
  - Adds decoding complexity.
  - Negligible savings for typical log sizes.

### ❌ `u16` or `u8`
- **Pros**: Smaller overhead.
- **Cons**:
  - `u16`: Limits records to 64 KB (too restrictive for some use cases).
  - `u8`: Limits records to 256 bytes (unusable for most logs).

### ❌ No Length Prefix (Delimited Stream)
- **Pros**: No overhead.
- **Cons**:
  - Impossible to parse without knowing record boundaries.
  - Not streamable (requires full file to be read first).
  - Vulnerable to corruption (one bad byte breaks the entire stream).

---

## Conclusion

**Using `u32` (big-endian) for record length is the best choice for Lulu-Logs because:**

1. ✅ **Aligns with FlatBuffers' design** (no varint support).
2. ✅ **Sufficient for all practical use cases** (up to 4 GB per record).
3. ✅ **Simple and fast** (fixed-size, no decoding overhead).
4. ✅ **Cross-platform** (big-endian ensures consistency).
5. ✅ **Future-proof** (can switch to `u64` if needed).

> **Varint would add complexity for negligible benefits.**

---

*Created for Lulu-Logs v2.0.0 - 2026-07-07*
