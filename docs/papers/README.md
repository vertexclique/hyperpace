# papers

One folder per research invocation, named for the topic:

```
papers/<slug>/
  README.md      topic, date, scope, the search log
  sources/       downloaded papers and primary facts, one file per source
  summaries/     one SUMMARY per source
  JUDGEMENT.md   the ranked verdict that feeds docs/plans/
```

Run it with: vertexia research <topic>. The research is unbounded; stop at
saturation. Every claim in JUDGEMENT.md cites a file in sources/ or a tier-1
source. See the vertexia-research skill.
