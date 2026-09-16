# infra

The infrastructure change log. Every infra change applied to this repo is
recorded here so the how is always recoverable: what changed, the exact commands,
the env vars and their checks, the plan that was applied, the verification, and
the rollback.

- One dated entry per change: infra/<YYYY-MM-DD>-<slug>.md, from
  templates/infra-change.md. Newest first.
- This is not optional. Never apply an infra change you cannot document. A
  change whose how is not written down did not happen safely.

Scaffold a new entry with: make infra-log (or copy the template). Apply changes
with: make tf-plan then make tf-apply, which check the env and remind you to log.
