# Infra change: <what> (<YYYY-MM-DD>)

Repo: <repo>. Applied by: <who>. Status: applied | rolled-back.

- What changed: <the resource, config, or env that changed, and why>
- How applied: <the exact commands, in order, copy-pasteable>
- Env vars: <each var read or set; the check that proves each is present and
  correct before apply, for example: test -n "$AWS_REGION", terraform validate>
- Plan: <link to the saved terraform plan or the diff that was applied>
- Verification: <the command run after apply and the observed result that proves
  it took, not an assumption>
- Rollback: <the exact steps to undo this change>
- Notes: <ordering, dependencies, gotchas, blast radius>

Keep one dated entry per change in docs/infra/, newest first. Never apply an
infra change you cannot document, and never skip this log.
