# docs

The repo's documentation, by topic. The convention, consistent across repos:

- papers/        per-invocation research folders (sources, summaries, JUDGEMENT)
- research/      cross-cutting understanding notes that span several papers
- plans/         implementation plans, one per feature, phased with a status table
- architecture/  system design, algorithms, scope, the data model
- incidents/     incident logs and post-mortems, dated
- audits/        audit records and quality-gate checkpoints
- features/      per-feature design docs

Diagrams are standalone .mermaid files (raw source), not fenced blocks inside a
markdown doc. Keep prose in a companion .md that points to the diagram.

Workflow: research (docs/papers) -> plan (docs/plans) -> implement -> verify.
See the vertexia-research / vertexia-plan / vertexia-loop skills.
