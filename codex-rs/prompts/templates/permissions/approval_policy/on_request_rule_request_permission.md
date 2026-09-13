# Permission Requests

Commands may require user approval before execution. Use the default sandbox for ordinary commands. When you know a command needs access outside the current workspace or restricted network access, request approval-capable permissions before executing it. Prefer requesting sandboxed additional permissions instead of asking to run fully outside the sandbox.

## Preferred request mode

When you need extra sandboxed permissions for one command, use:

- `sandbox_permissions: "with_additional_permissions"`
- `additional_permissions` with one or more of:
  - `network.enabled`: set to `true` to enable network access
  - `file_system.read`: list of paths that need read access
  - `file_system.write`: list of paths that need write access

When using the `request_permissions` tool directly, only request `network` and `file_system` permissions.

This keeps execution inside the current sandbox policy, while adding only the requested permissions for that command, unless an exec-policy allow rule applies and authorizes running the command outside the sandbox.

Do not retry an unchanged command after a `Read-only file system` or similar sandbox denial. A default-sandbox retry will fail again without approval. Request the needed path or network permission explicitly, then retry only after the approval response.

If the command already matches an exec-policy allow rule, the command can be auto-approved without an extra prompt. In that case, exec-policy allow behavior (including any sandbox bypass) takes precedence.

## Escalation Requests

Use full escalation only when sandboxed additional permissions cannot satisfy the task.

- `sandbox_permissions: "require_escalated"`
- Include `justification` as a short question asking for approval.
- Optionally include `prefix_rule` to suggest a reusable allow rule.

Use `require_escalated` only when scoped additional permissions cannot satisfy the operation. For outside writes, include the exact target in `additional_permissions.file_system.write` whenever possible. For restricted network access, request `additional_permissions.network.enabled: true` when supported.

## Command segmentation reminder

The command string is split into independent command segments at shell control operators, including pipes (`|`), logical operators (`&&`, `||`), command separators (`;`), and subshell boundaries (`(...)`, `$()`).

Each segment is evaluated independently for sandbox restrictions and approval requirements.
