# Snapshots

Snapshots capture the structured output of an action and compare future runs against a stored baseline to detect regressions.

They are optional but strongly recommended for non-trivial logic.

---

## What a Snapshot Is

A snapshot is the **full JSON output envelope** produced by a successful action run.

It includes:

- `ok` status
- `meta` (timings, memory, fixture context)
- `output` (the action’s returned payload)
- `failures` (if present)

Snapshots are compared **byte-for-byte at the JSON structure level**, not textually.

---

## Enabling Snapshots

Snapshots can be enabled in three ways:

### Via `config.yaml`

```yaml
snapshots:
  enabled: true
```

---

### Via CLI

```bash
hsemulate run --snapshot
```

This forces `snapshots.enabled = true` for the run.

---

### Automatically in CI Mode

```bash
hsemulate test
```

CI mode always enables snapshots.

---

## Snapshot Storage Location

Snapshots are stored in the `snapshots/` directory at the project root.

Example:

```text
snapshots/
└── <snapshot-key>.json
```

The directory is created automatically if it does not exist.

---

## Snapshot Key Generation

Each snapshot is uniquely keyed by:

- The canonical action file path
- The fixture path

This means:

- Each action + fixture pair has its own snapshot
- Different fixtures never share snapshots
- Changing the action file path results in a new snapshot

---

## Baseline Creation

On the **first run** with snapshots enabled:

- If no snapshot exists:

  - The current output is written as the baseline
  - The run passes

- No comparison is performed

This allows snapshots to be adopted incrementally.

---

## Snapshot Comparison

On subsequent runs:

- The current output is compared against the stored snapshot
- Any difference causes a failure

Example failure:

```
Snapshot mismatch (snapshots/abc123.json): value changed at output.result.total
```

---

## Snapshot Comparison Rules

- Comparison is structural (parsed JSON)
- Ordering differences are detected
- Missing or additional fields cause failure
- All differences are treated as regressions

There is no fuzzy or tolerance-based comparison.

---

## Ignore Rules (Current Status)

`config.yaml` supports an `ignore` field:

```yaml
snapshots:
  enabled: true
  ignore:
    - output.timestamp
    - meta.runId
```

However, **ignore rules are not yet applied during comparison**.

They are reserved for future implementation and currently have no effect.

You should treat all snapshot comparisons as strict.

---

## Interaction With Repeats

When `repeat > 1`:

- The snapshot baseline is created from the **first run**
- Subsequent runs are compared against that baseline
- Any variation between repeats causes failure

This makes snapshots a powerful flaky-behaviour detector.

---

## Interaction With Assertions and Budgets

Snapshot comparison occurs:

1. After action execution
2. After assertions
3. After budgets
4. Before output emission

A snapshot mismatch is treated the same as an assertion or budget failure.

---

## Updating Snapshots

There is no automatic snapshot update mode.

To update snapshots intentionally:

1. Delete the relevant snapshot file(s)
2. Re-run with snapshots enabled
3. New baselines will be created

This makes snapshot updates explicit and deliberate.

---

## When to Use Snapshots

Use snapshots when:

- Output is complex or nested
- Multiple fields change together
- You want regression protection without many assertions

Avoid snapshots when:

- Output contains unavoidable non-determinism
- You only care about a small number of invariants

In those cases, prefer assertions.

---

## Summary

- Snapshots are strict and deterministic
- Baselines are created automatically
- Comparisons are structural and exact
- Ignore rules are planned but not yet enforced
- Snapshot mismatches are fatal

---
# Promotion (`/promote`)

The `/promote` endpoint updates an existing **HubSpot CUSTOM_CODE workflow action** with tested source code.

It is designed to be:

* Deterministic
* Safe by default
* Fully automatable from CI or a UI
* Compatible with `hsemulate test` + snapshot gating

Promotion **does not create workflows or actions**.
It updates an existing action in place.

---

## What Promotion Does

A promotion performs the following steps:

1. Receives raw action source code (JS or Python)
2. Computes a canonical SHA-256 hash of the source
3. Injects a `hsemulator-sha` marker comment
4. Fetches the target HubSpot workflow
5. Locates the target `CUSTOM_CODE` action by selector
6. Applies drift protection
7. Updates the action source (and runtime if provided)
8. Writes the updated workflow back to HubSpot

All steps are atomic from the caller’s perspective.

---

## Drift Protection

Promotion uses a **hash marker** to ensure ownership and prevent accidental overwrites.

Injected marker (example):

```js
// hsemulator-sha: a3f4c1...
```

or (Python):

```py
# hsemulator-sha: a3f4c1...
```

Rules:

* If the existing action has the same hash → **no-op**
* If the existing hash differs → update proceeds
* If no marker exists:

  * Promotion fails by default
  * `force: true` overrides this protection

This prevents overwriting manually-edited or externally-managed actions.

---

## Endpoint

```
POST /promote
```

This endpoint is protected by the runtime API key middleware.

---

## Request Body

```json
{
  "hubspot_token": "pat-xxxx",
  "workflow_id": "123456789",
  "selector": {
    "type": "secret",
    "value": "HUBSPOT_PRIVATE_APP_TOKEN"
  },
  "runtime": "nodejs18.x",
  "source_code": "// action source here",
  "force": false,
  "dry_run": false
}
```

---

## Request Fields

### `hubspot_token` (required)

HubSpot **private app token** used to fetch and update the workflow.

* Must belong to the portal containing the workflow
* Must have automation/workflow scopes

This token is **not stored** by the runtime.

---

### `workflow_id` (required)

The HubSpot workflow ID containing the target action.

This must be a valid workflow accessible by the token.

---

### `selector` (required)

Identifies the `CUSTOM_CODE` action to update.

Currently supported selector:

```json
{
  "type": "secret",
  "value": "HUBSPOT_PRIVATE_APP_TOKEN"
}
```

This matches against `action.secretNames[]`.

Rules:

* Exactly **one** matching action must be found
* Multiple matches cause failure
* Zero matches cause failure

---

### `runtime` (optional)

Overrides the action runtime in HubSpot.

Example values:

* `nodejs18.x`
* `python3.11`

If omitted, the existing runtime is preserved.

---

### `source_code` (required)

Raw JavaScript or Python source code to promote.

* Hash marker is injected automatically
* Existing marker is replaced if present

---

### `force` (optional, default `false`)

Overrides safety checks.

Effects:

* Allows overwriting actions without a hash marker
* Skips drift ownership protection

Use with caution.

---

### `dry_run` (optional, default `false`)

If `true`:

* No PUT request is sent to HubSpot
* Full validation and hashing still occur
* A summary response is returned

Recommended for CI validation and previews.

---

## Responses

### Success (Dry Run)

```json
{
  "ok": true,
  "dry_run": true,
  "workflow_id": "123456789",
  "hash": "a3f4c1...",
  "action_index": 4
}
```

---

### Success (Promotion Applied)

```json
{
  "ok": true,
  "workflow_id": "123456789",
  "hash": "a3f4c1...",
  "revision_id": "987654321"
}
```

---

### No-Op (Already Up To Date)

```json
{
  "ok": true,
  "status": "noop",
  "hash": "a3f4c1..."
}
```

---

### Failure Examples

**Unauthorized (API key):**

```json
{
  "ok": false,
  "error": "Unauthorized"
}
```

**Invalid selector:**

```json
{
  "ok": false,
  "error": "Only selector.type = 'secret' is supported"
}
```

**Workflow fetch failure:**

```json
{
  "ok": false,
  "error": "HubSpot GET flow failed: 400 Bad Request Invalid request"
}
```

---

## Interaction With Tests and Snapshots

The runtime `/promote` endpoint itself does **not** execute tests.

However, it is designed to be called **only after**:

* `hsemulate test`
* Snapshot comparison
* Budget enforcement
* Assertion validation

The CLI `hsemulate promote` command enforces these gates automatically.

When using `/promote` directly, the caller is responsible for enforcing test discipline.

---

## Safety Guarantees

* No workflow creation
* No action creation
* Deterministic selection
* Ownership protection via hash
* Explicit override required for unsafe writes

Promotion is intentionally strict.