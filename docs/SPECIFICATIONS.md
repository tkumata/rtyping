# 仕様 (SPECIFICATIONS)

## テストケース詳細

### `src/usecase/wpm.rs`

| ID | Test case name | Input (len, sec, miss) | Expected value (approx) | Remarks |
| ---- | ---- | ---- | ---- | ---- |
| WPM-001 | Normal: Standard input | 50, 60.0, 0 | 10.0 | (50 - 0) / (5 * 60/60) = 50 / 5 = 10 |
| WPM-002 | Normal: With miss | 50, 60.0, 5 | 9.0 | (50 - 5) / 5 = 9 |
| WPM-003 | Boundary: Short time | 10, 10.0, 0 | 12.0 | 10 / (5 * 10/60) = 10 / (50/60) = 10 / 0.833... = 12 |
| WPM-004 | Abnormal: Zero time | 10, 0.0, 0 | INFINITY | Division by zero |

### `src/domain/entity.rs`

| ID | Test case name | Expected behavior | Remarks |
| ---- | ---- | ---- | ---- |
| ENT-001 | Sample text retrieval success | Returns `Ok` and ensures the content is not empty | Confirmation of `include_str!` behavior |

### `src/usecase/generate_sentence.rs`

| ID | Test case name | Input (level) | Expected behavior | Remarks |
| ---- | ---- | ---- | ---- | ---- |
| GEN-001 | Generation success | 10 | Returns `Ok` and ensures the content is not empty | Content is random |
