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

### `GitHub Actions`

| ID | Test case name | Trigger | Expected behavior | Remarks |
| ---- | ---- | ---- | ---- | ---- |
| CI-001 | Push test execution | push to `main` | `test.yml` runs `cargo test` on Ubuntu and macOS | Stops on failure |
| CI-002 | Pull request test execution | pull_request | `test.yml` runs `cargo test` on Ubuntu and macOS | Prevents unverified merge |
| CI-003 | Release guard | push changing `Cargo.toml` on `main` | `version-check.yml` executes tests before release build | Release job depends on test job |

## CLI Default Behavior

| ID | Test case name | Input | Expected behavior | Remarks |
| ---- | ---- | ---- | ---- | ---- |
| CLI-001 | Default launch is silent | no args | BGM does not start and typing feedback sound does not play | `sound` flag is false |
| CLI-002 | Sound flag enables all sound output | `--sound` | BGM starts and typing feedback sound plays on correct input | Uses existing `freq` value for feedback tone |
| CLI-003 | Default timeout | no args | Timer limit is 60 seconds | Value exposed through help and runtime |

## API Generation and Config

| ID | Test case name | Input | Expected behavior | Remarks |
| ---- | ---- | ---- | ---- | ---- |
| CLI-004 | Google provider selection | `--google` | Startup provider becomes Google AI Studio | Local generator is not used |
| CLI-005 | Groq provider selection | `--groq` | Startup provider becomes Groq | Local generator is not used |
| CLI-006 | Exclusive provider flags | `--google --groq` | Startup fails with argument error | Mutually exclusive |
| CFG-001 | Config file save | Save from Config screen | `~/.config/rtyping/config.json` is created | Stores provider entries |
| CFG-002 | API key encryption | Save API key | Config file does not contain the plain API key | Ciphertext and nonce are stored |
| CFG-003 | Config restore | Restart after save | URL, model, and decrypted API key return to the form | Requires `config.key` |
| UI-001 | Menu navigation | `Up`, `Down`, `Enter` on title screen | `Start Game` and `Config` can be selected | Initial focus is `Start Game` |
| UI-002 | Config input edit | Character input and `Backspace` | Focused field value changes | Supports both Google and Groq sections |
| UI-003 | Config save action | `Enter` on Config screen | Current values are persisted and success message is shown | `Esc` returns without saving |
| API-001 | Google generation | `--google` with valid config | Text is generated from Google AI Studio | Uses configured URL, model, and API key |
| API-002 | Groq generation | `--groq` with valid config | Text is generated from Groq | Uses configured URL, model, and API key |
| API-003 | Missing config handling | `--google` or `--groq` without saved config | App returns to menu with error message | No panic |
| API-004 | Network failure handling | Provider request fails | App returns to menu with error message | No panic |
| LEN-001 | Level based generation length | `--level 20` | Generated text length is trimmed to about `level * 5` chars | Same rule for local and remote generation |
| LEN-002 | Local fallback behavior | no provider flag | Existing local generator still works | Backward compatibility |

Google request URL rule:
- Build the final URL as `API URL/` + `Model` + `:generateContent`
- Append the API key as the `key` query parameter
